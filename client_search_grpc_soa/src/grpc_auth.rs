use core::fmt::Debug;
use core::str::FromStr;
use std::collections::HashSet;
use std::future::Future;
use std::pin::Pin;
use http::{
    HeaderName, HeaderValue,
    // request::Builder,
};
use log::{debug, error};
use tonic::{
    Request, Status,
    // codegen::BoxFuture,
    // metadata::{Ascii, KeyAndValueRef, MetadataValue, ValueRef},
    metadata::KeyAndValueRef,
};
use tonic_async_interceptor::AsyncInterceptor;
use tower::BoxError;
use mvv_auth::{
    //AuthUserProvider,
    backend::{RequestAuthenticated, authz_backend::AuthorizeBackend},
    //permission::{PermissionProvider, PermissionSet},
};
use mvv_auth::backend::UserContext;
use mvv_common::string::StaticRefOrString;
//--------------------------------------------------------------------------------------------------



#[derive(Debug, Clone)]
pub struct GrpcAuthInterceptor <
    Usr: axum_login::AuthUser + Clone + 'static,
    // Perm: Debug + Clone + Send + Sync + 'static,
    // PermSet: PermissionSet + Clone + 'static,
    AuthB: AuthorizeBackend<User=Usr> + RequestAuthenticated<User=Usr> + Debug + 'static,
> {
    pub public_endpoints: HashSet<StaticRefOrString>,
    // pub user_provider: Arc<dyn AuthUserProvider<User=Usr> + Send + Sync + 'static>,
    // pub permission_provider: Arc<dyn PermissionProvider<User=Usr,Permission=Perm,PermissionSet=PermSet> + Send + Sync + 'static>,
    pub auth: AuthB
}


fn get_end_point_path<T>(req: &Request<T>) -> anyhow::Result<String> {
    let url = get_grpc_req_url(req) ?;
    Ok(url.path().to_owned())
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
    // Perm: Debug + Clone + Send + Sync + 'static,
    // PermSet: PermissionSet + Clone + 'static,
    AuthB: AuthorizeBackend<User=Usr> + RequestAuthenticated<User=Usr> + Debug + 'static,
> AsyncInterceptor for GrpcAuthInterceptor<Usr, /*Perm, PermSet,*/ AuthB> {
    type Future = Pin<Box<dyn Future<Output=Result<Request<()>, Status>> + Send + 'static >>;
    // type Future = tonic::codegen::BoxFuture<Request<()>, Status>;
    // type Future = futures_util::future::BoxFuture<Request<()>, Status>;

    fn call(&mut self, request: Request<()>) -> Self::Future {
        Box::pin(authenticate_req::<Usr, /*Perm, PermSet,*/ AuthB>(self.clone(), request))
    }
}



async fn authenticate_req <
    Usr: axum_login::AuthUser + Clone + 'static,
    // Perm: Debug + Clone + Send + Sync + 'static,
    // PermSet: PermissionSet + Clone + 'static,
    AuthB: AuthorizeBackend<User=Usr> + RequestAuthenticated<User=Usr> + Debug + 'static,
> (
    state: GrpcAuthInterceptor<Usr,/*Perm,PermSet,*/AuthB>,
    req: Request<()>,
    ) -> Result<Request<()>, Status> {

    let end_point_path = get_end_point_path(&req)
        .map_err(|err|{
            error!("No end-point URL in grpc request [{err:?}]");
            Status::internal("Auth error")
        }) ?;

    let full_end_point_path: &str = end_point_path.as_str();
    if state.public_endpoints.contains(full_end_point_path) {
        return Ok(req);
    }

    let end_point_service_path = full_end_point_path.rsplit_once('/');
    if let Some((serv_path, _method)) = end_point_service_path {
        if state.public_endpoints.contains(serv_path) {
            return Ok(req);
        }
    }

    let res = auth_req_impl::<Usr, /*Perm, PermSet,*/ AuthB>(state, req).await;
    match res {
        Ok(Some(user)) => {
            debug!("User [{user:?}] is authenticated.");
            Ok(Request::new(()))
        }
        Ok(None) => {
            Err(Status::unauthenticated("Unauthenticated"))
        }
        Err(err) => {
            error!("Authentication error: {err:?}");
            Err(Status::unauthenticated("Unauthenticated"))
        }
    }
}

async fn auth_req_impl <
    Usr: axum_login::AuthUser + Clone + 'static,
    // Perm: Debug + Clone + Send + Sync + 'static,
    // PermSet: PermissionSet + Clone + 'static,
    AuthB: AuthorizeBackend<User=Usr> + RequestAuthenticated<User=Usr> + Debug + 'static,
> (
    auth_state: GrpcAuthInterceptor<Usr,/*Perm,PermSet,*/AuthB>,
    mut grpc_req: Request<()>,
) -> anyhow::Result<Option<Usr>> {

    let axum_req= grpc_req_to_axum_req(&grpc_req) ?;

    let (_req, res) = auth_state.auth.do_authenticate_request::<AuthB, ()>(
        None, axum_req).await;

    match res {
        Ok(Some(user)) => {
            grpc_req.extensions_mut().insert(UserContext::new(user.clone()));
            Ok(Some(user))
        }
        Ok(None) =>
            Ok(None),
        Err(err) => {
            error!("Grpc AuthBackend error: {err:?}");
            Err(err.into())
        }
    }
}

/*
fn grpc_req_to_http_req_parts<T>(grpc_req: &Request<T>) -> anyhow::Result<http::request::Parts> {
    let mut parts_b = Builder::default()
        // .method(request.metadata().url)
        ;
    let mut req: http::Request<()> = parts_b.body(()) ?;
    let (mut parts, ()) = req.into_parts();

    for ref k_v_ref in grpc_req.metadata().iter() {
        let (header_name, header_value) = to_http_header(k_v_ref) ?;
        parts.headers.insert(header_name, header_value);
    }

    Ok(parts)
}
*/

fn grpc_req_to_axum_req<T>(grpc_req: &Request<T>) -> anyhow::Result<axum::extract::Request<axum::body::Body>> {
    let mut axum_req: axum::extract::Request<axum::body::Body> =
        axum::extract::Request::new(axum::body::Body::empty());

    for ref k_v_ref in grpc_req.metadata().iter() {
        let (header_name, header_value) = to_http_header(k_v_ref) ?;
        axum_req.headers_mut().insert(header_name, header_value);
    }

    Ok(axum_req)
}

fn to_http_header(kv_ref: &KeyAndValueRef) -> anyhow::Result<(HeaderName, HeaderValue)> {
    let (header_name, header_value) = match kv_ref {
        KeyAndValueRef::Ascii(k, v) => {
            let header_name = HeaderName::from_str(k.as_str())
                /*.map_err(|_|Status::internal("???"))*/ ?; // TODO: log warn and ignore it?
            let header_value: HeaderValue = HeaderValue::from_bytes(v.as_bytes())
                /*.map_err(|_|Status::internal("Bla-bla"))*/ ?;
            (header_name, header_value)
        }
        KeyAndValueRef::Binary(k, v) => {
            let header_name = HeaderName::from_str(k.as_str())
                /*.map_err(|_|Status::internal("???"))*/ ?; // TODO: log warn and ignore it?
            let header_value: HeaderValue = HeaderValue::from_bytes(v.as_encoded_bytes())
                /*.map_err(|_|Status::internal("Bla-bla"))*/ ?;
            (header_name, header_value)
        }
    };
    Ok((header_name, header_value))
}



#[derive(Debug, Clone)]
#[allow(dead_code)]
struct GrpcReqEnrichPredicate;

impl <Body> tower::filter::Predicate<http::request::Request<Body>> for GrpcReqEnrichPredicate {
    type Request = http::request::Request<Body>;

    fn check(&mut self, mut req: Self::Request) -> Result<Self::Request, BoxError> {

        let ext_uri = req.extensions().get::<axum::extract::OriginalUri>();
        if ext_uri.is_none() {
            let uri = req.uri().clone();
            req.extensions_mut().insert(axum::extract::OriginalUri(uri));
        }
        Ok(req)
    }
}

// TODO: temporary pub, add extension trait for installing it
pub fn grpc_req_enrich<Body>(mut req: http::request::Request<Body>)
                         -> Result<http::request::Request<Body>, BoxError> {
    let ext_uri = req.extensions().get::<axum::extract::OriginalUri>();
    if ext_uri.is_none() {
        let uri = req.uri().clone();
        req.extensions_mut().insert(axum::extract::OriginalUri(uri));
    }
    Ok(req)
}
