use std::fmt::{Debug, Display};
use std::hash::Hash;
use askama_axum::IntoResponse;
use axum::body::Body;
use axum::extract::{Request, State};
use http::StatusCode;
use log::error;

use crate::backend::{
    AuthnBackendAttributes, RequestAuthenticated, authz_backend::AuthorizeBackend,
};
use crate::permission::{ PermissionSet, util::log_unauthorized_access };



pub async fn validate_authentication_chain <
    Usr: axum_login::AuthUser,
    Creds: Send + Sync,
    Err: std::error::Error + IntoResponse + Send + Sync,
    Backend: axum_login::AuthnBackend<User=Usr,Credentials=Creds,Error=Err> + RequestAuthenticated + 'static
> (
    auth_session: axum_login::AuthSession<Backend>,
    req: Request,
    next: axum::middleware::Next,
) -> http::Response<Body> {

    let (req, is_auth_res) =
        auth_session.backend.do_authenticate_request::
            <Creds, Err, Backend,()>(auth_session.clone(), req).await;
    match is_auth_res {
        Ok(None) => StatusCode::UNAUTHORIZED.into_response(),
        Ok(_) => next.run(req).await,
        Err(action) => action.into_response()
    }
}


pub async fn validate_authorization_chain<
    Usr: axum_login::AuthUser + 'static,
    Creds: Send + Sync,
    Err: std::error::Error + IntoResponse + Send + Sync,
    Backend: axum_login::AuthnBackend<User=Usr,Credentials=Creds,Error=Err>
        + RequestAuthenticated
        + AuthorizeBackend<User=Usr,Permission=Perm,PermissionSet=PermSet>
        + AuthnBackendAttributes
        + 'static,
    Perm: Hash + Eq + Display + Debug + Clone + Send + Sync + 'static,
    PermSet: PermissionSet<Permission=Perm> + Clone + Display + 'static,
> (
    auth_session: axum_login::AuthSession<Backend>,
    required_permissions: PermSet,
    req: Request,
    next: axum::middleware::Next,
) -> http::Response<Body> {

    let (req, user_opt_res) = auth_session.backend
        .do_authenticate_request::
            <Creds, Err, Backend, ()>(auth_session.clone(), req).await;

    let user_opt = match user_opt_res {
        Ok(user_opt) => user_opt,
        Err(err_response) => { return err_response.into_response(); }
    };

    if let Some(ref user) = user_opt {
        let authz_res = auth_session.backend.authorize(user, required_permissions).await;
        match authz_res {
            Ok(ref authz_res) => {
                if authz_res.is_authorized() {
                    next.run(req).await
                } else {
                    log_unauthorized_access(req, user, authz_res);
                    // Probably by security reason it would be better return 401/404. It is up to you.
                    StatusCode::FORBIDDEN.into_response()
                }
            }
            Err(err) => {
                // TODO: verify printed result and stack-trace
                error!("Permission process error: {}", err);
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    } else {
        auth_session.backend.propose_authentication_action(&req)
            .map(|unauthenticated_action|unauthenticated_action.into_response())
            .unwrap_or_else(||StatusCode::UNAUTHORIZED.into_response())
    }
}


#[inline]
pub async fn validate_authorization_chain_as_middleware_fn<
    Usr: axum_login::AuthUser + 'static,
    Creds: Send + Sync,
    Err: std::error::Error + IntoResponse + Send + Sync,
    Backend: axum_login::AuthnBackend<User=Usr,Credentials=Creds,Error=Err>
        + RequestAuthenticated
        + AuthorizeBackend<User=Usr,Permission=Perm,PermissionSet=PermSet>
        + AuthnBackendAttributes
        + 'static,
    Perm: Hash + Eq + Display + Debug + Clone + Send + Sync + 'static,
    PermSet: PermissionSet<Permission=Perm> + Clone + Display + 'static,
> (
    auth_session: axum_login::AuthSession<Backend>,
    required_permissions: State<PermSet>,
    req: Request, next: axum::middleware::Next,
) -> http::Response<Body> {
    validate_authorization_chain(auth_session, required_permissions.0, req, next).await
}
