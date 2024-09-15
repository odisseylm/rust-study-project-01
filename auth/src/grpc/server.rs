use core::{
    fmt::Debug,
    str::FromStr,
};
use std::{
    collections::HashMap,
    future::Future,
    pin::Pin,
    sync::Arc,
};
use http::{
    HeaderName, HeaderValue,
    // request::Builder,
};
use log::{debug, error, warn};
use tonic::{
    Request, Status,
    // codegen::BoxFuture,
    // metadata::{Ascii, KeyAndValueRef, MetadataValue, ValueRef},
    metadata::KeyAndValueRef,
};
use tonic_async_interceptor::AsyncInterceptor;
use tower::BoxError;
use mvv_common::grpc::TonicErrToStatusExt;
use crate::{
    //AuthUserProvider,
    backend::{RequestAuthenticated, authz_backend::AuthorizeBackend, UserContext},
    permission::PermissionSet,
};
use mvv_common::string::StaticRefOrString;
//--------------------------------------------------------------------------------------------------



#[derive(Debug, Clone)]
pub struct GrpcAuthzInterceptor<
    Usr: axum_login::AuthUser + Clone + 'static,
    AuthB: AuthorizeBackend<User=Usr> + RequestAuthenticated<User=Usr> + Debug + 'static,
> {
    pub endpoints_roles: Arc<HashMap<StaticRefOrString, AuthB::PermissionSet>>,
    pub auth: Arc<AuthB>,
}


#[inline]
pub fn public_access_permissions<PermSet: PermissionSet>() -> PermSet { PermSet::new() }

// Feel free to add any other standard PUBLIC services.
// !!! But make sure that it is unsafe in context of your application/company !!!
#[inline]
pub fn predefined_public_endpoints_roles <PermSet: PermissionSet>() -> HashMap<StaticRefOrString, PermSet> {
    HashMap::from([
        ("/grpc.reflection.v1.ServerReflection".into(), public_access_permissions()),
        ("/grpc.health.v1.Health".into(), public_access_permissions()),
    ])
}

fn get_end_point_path<T>(req: &Request<T>) -> anyhow::Result<String> {
    let url = get_grpc_req_url(req) ?;
    Ok(url.path().to_owned())
}
fn get_end_point_path_with_grpc_err<T>(req: &Request<T>) -> Result<String, Status> {
    let endpoint_path = get_end_point_path(&req)
        .to_tonic_internal_err("Error of getting endpoint url") ?;
    Ok(endpoint_path)
}

pub fn get_grpc_req_url<T>(req: &Request<T>) -> anyhow::Result<http::uri::Uri> {
    let uri = req.extensions().get::<axum::extract::OriginalUri>();
    match uri {
        None => anyhow::bail!("Request does not contains url info."),
        Some(uri) => Ok(uri.0.clone())
    }
}

impl <
    Usr: axum_login::AuthUser + Clone + 'static,
    AuthB: AuthorizeBackend<User=Usr> + RequestAuthenticated<User=Usr> + Debug + 'static,
> GrpcAuthzInterceptor<Usr, /*Perm, PermSet,*/ AuthB> {

    fn find_end_point_path_roles(&self, req: &Request<()>) -> Result<Option<&AuthB::PermissionSet>, Status> {

        let end_point_path = get_end_point_path_with_grpc_err(&req) ?;

        let full_end_point_path: &str = end_point_path.as_str();
        let end_point_path_roles = self.endpoints_roles.get(full_end_point_path);
        if end_point_path_roles.is_some() {
            return Ok(end_point_path_roles);
        }

        let end_point_service_path = full_end_point_path.rsplit_once('/');
        let roles =
            if let Some((serv_path, _method)) = end_point_service_path {
                self.endpoints_roles.get(serv_path)
            } else {
                None
            };

        Ok(roles)
    }
}

impl <
    Usr: axum_login::AuthUser + Clone + 'static,
    AuthB: AuthorizeBackend<User=Usr> + RequestAuthenticated<User=Usr> + Debug + 'static,
> AsyncInterceptor for GrpcAuthzInterceptor<Usr, /*Perm, PermSet,*/ AuthB> {
    type Future = Pin<Box<dyn Future<Output=Result<Request<()>, Status>> + Send + 'static >>;
    // type Future = tonic::codegen::BoxFuture<Request<()>, Status>;
    // type Future = futures_util::future::BoxFuture<Request<()>, Status>;

    fn call(&mut self, request: Request<()>) -> Self::Future {
        Box::pin(authorize_req::<Usr, /*Perm, PermSet,*/ AuthB>(self.clone(), request))
    }
}



async fn authorize_req <
    Usr: axum_login::AuthUser + Clone + 'static,
    AuthB: AuthorizeBackend<User=Usr> + RequestAuthenticated<User=Usr> + Debug + 'static,
> (
    state: GrpcAuthzInterceptor<Usr,/*Perm,PermSet,*/AuthB>,
    req: Request<()>,
    ) -> Result<Request<()>, Status> {

    let required_roles = state.find_end_point_path_roles(&req) ?;
    let required_roles = match required_roles {
        None => {
            let endpoint_path = get_end_point_path_with_grpc_err(&req) ?;
            warn!("Authorization configuration error for [{endpoint_path}]");
            return Err(Status::permission_denied("Unauthorized"));
        }
        Some(required_roles) =>
            required_roles.clone(),
    };

    if required_roles.is_empty() {
        return Ok(req);
    }

    let res = authz_req_impl::<Usr, /*Perm, PermSet,*/ AuthB>(state, req, required_roles.clone()).await;
    match res {
        Ok(AuthorizeResult::Authorized { user }) => {
            debug!("User [{user:?}] is authorized.");
            Ok(Request::new(()))
        }
        Ok(AuthorizeResult::Unauthorized { user }) => {
            debug!("User [{user:?}] is unauthorized.");
            Err(Status::permission_denied("Unauthorized"))
        }
        Ok(AuthorizeResult::Unauthenticated) => {
            debug!("User is authorized.");
            Err(Status::unauthenticated("Unauthenticated"))
        }
        Err(err) => {
            error!("Authentication error: {err:?}");
            Err(Status::unauthenticated("Unauthenticated"))
        }
    }
}

enum AuthorizeResult <
    Usr: axum_login::AuthUser + Clone + 'static,
> {
    Authorized { user: Usr },
    Unauthorized { user: Usr },
    Unauthenticated,
}


async fn authz_req_impl <
    Usr: axum_login::AuthUser + Clone + 'static,
    AuthB: AuthorizeBackend<User=Usr> + RequestAuthenticated<User=Usr> + Debug + 'static,
> (
    auth_state: GrpcAuthzInterceptor<Usr,/*Perm,PermSet,*/AuthB>,
    mut grpc_req: Request<()>,
    required_roles: AuthB::PermissionSet,
) -> anyhow::Result<AuthorizeResult<Usr>> {

    let axum_req= grpc_req_to_axum_req(&grpc_req) ?;

    let (_req, res) = auth_state.auth.do_authenticate_request::<AuthB, ()>(
        None, axum_req).await;

    match res {
        Ok(Some(user)) => {
            let has_permissions = auth_state.auth.has_permissions(&user, required_roles).await ?;
            if has_permissions {
                grpc_req.extensions_mut().insert(UserContext::new(user.clone()));
                Ok(AuthorizeResult::Authorized {user })
            } else {
                Ok(AuthorizeResult::Unauthorized { user })
            }
        }
        Ok(None) =>
            Ok(AuthorizeResult::Unauthenticated),
        Err(err) => {
            error!("Grpc AuthBackend error: {err:?}");
            Err(err.into())
        }
    }
}


fn grpc_req_to_axum_req<T>(grpc_req: &Request<T>) -> anyhow::Result<axum::extract::Request<axum::body::Body>> {
    let mut axum_req: axum::extract::Request<axum::body::Body> =
        axum::extract::Request::new(axum::body::Body::empty());

    for ref k_v_ref in grpc_req.metadata().iter() {
        let header_name_value = to_http_header(k_v_ref);
        match header_name_value {
            Ok((header_name, header_value)) => {
                axum_req.headers_mut().insert(header_name, header_value);
            }
            Err(err) => {
                warn!("Error converting header [{k_v_ref:?}]: {err:?}");
            }
        }
    }

    Ok(axum_req)
}

fn to_http_header(kv_ref: &KeyAndValueRef) -> anyhow::Result<(HeaderName, HeaderValue)> {
    let (header_name, header_value) = match kv_ref {
        KeyAndValueRef::Ascii(k, v) => {
            let header_name = HeaderName::from_str(k.as_str()) ?;
            let header_value: HeaderValue = HeaderValue::from_bytes(v.as_bytes()) ?;
            (header_name, header_value)
        }
        KeyAndValueRef::Binary(k, v) => {
            let header_name = HeaderName::from_str(k.as_str()) ?;
            let header_value: HeaderValue = HeaderValue::from_bytes(v.as_encoded_bytes()) ?;
            (header_name, header_value)
        }
    };
    Ok((header_name, header_value))
}



#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct GrpcReqEnrichPredicate;

impl <Body> tower::filter::Predicate<http::request::Request<Body>> for GrpcReqEnrichPredicate {
    type Request = http::request::Request<Body>;

    fn check(&mut self, req: Self::Request) -> Result<Self::Request, BoxError> {
        grpc_req_enrich(req)
    }
}


/// It can be used if you do not need real/specific type (in trait declaration)
/// For example: tonic::transport::Server.layer(FilterLayer::new(grpc_req_enrich))
///
pub fn grpc_req_enrich<Body>(mut req: http::request::Request<Body>)
    -> Result<http::request::Request<Body>, BoxError> {

    let ext_uri = req.extensions().get::<axum::extract::OriginalUri>();
    if ext_uri.is_none() {
        let uri = req.uri().clone();
        req.extensions_mut().insert(axum::extract::OriginalUri(uri));
    }
    Ok(req)
}


#[extension_trait::extension_trait]
pub impl<L> TonicServerGrpcReqEnrichExt<L> for tonic::transport::Server<L> {
    fn add_grpc_req_enrich_layer(self)
        -> tonic::transport::Server<tower_layer::Stack<
            tower::filter::FilterLayer<GrpcReqEnrichPredicate>, L>> {

        use tower::filter::FilterLayer;
        self.layer(FilterLayer::new(GrpcReqEnrichPredicate))
    }
}
