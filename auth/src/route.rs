use core::fmt::Display;
use askama_axum::IntoResponse;
use axum::{
    body::Body, extract::{Request, State},
};
use http::StatusCode;
use log::error;

use crate::{
    backend::{
        AuthnBackendAttributes, RequestAuthenticated,
        authz_backend::{AuthorizeBackend, PermissionProviderSource},
    },
    permission::util::log_unauthorized_access,
};
//--------------------------------------------------------------------------------------------------



pub async fn validate_authentication_chain <Backend> (
    auth_session: axum_login::AuthSession<Backend>,
    req: Request,
    next: axum::middleware::Next,
) -> http::Response<Body>
    where
        Backend: axum_login::AuthnBackend + RequestAuthenticated + 'static,
        Backend: axum_login::AuthnBackend<Error: IntoResponse>,
{
    let (req, is_auth_res) =
        auth_session.backend.do_authenticate_request::<Backend,()>(
            auth_session.clone(), req).await;
    match is_auth_res {
        Ok(None) => StatusCode::UNAUTHORIZED.into_response(),
        Ok(_) => next.run(req).await,
        Err(action) => action.into_response()
    }
}


pub async fn validate_authorization_chain <Backend> (
    auth_session: axum_login::AuthSession<Backend>,
    required_permissions: <Backend as PermissionProviderSource>::PermissionSet,
    req: Request,
    next: axum::middleware::Next,
) -> http::Response<Body>
    where
        Backend: axum_login::AuthnBackend<Error: IntoResponse>
            + RequestAuthenticated
            + AuthorizeBackend
            + AuthnBackendAttributes
            + 'static,
        Backend: axum_login::AuthnBackend<User = <Backend as PermissionProviderSource>::User>,
        <Backend as PermissionProviderSource>::Permission: Display,
        <Backend as PermissionProviderSource>::PermissionSet: Display,
{

    let (req, user_opt_res) = auth_session.backend
        .do_authenticate_request::<Backend, ()>(auth_session.clone(), req).await;

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
                error!("Permission process internal error: {err:?}");
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
pub async fn validate_authorization_chain_as_middleware_fn <Backend> (
    auth_session: axum_login::AuthSession<Backend>,
    required_permissions: State< <Backend as PermissionProviderSource>::PermissionSet >,
    req: Request, next: axum::middleware::Next,
) -> http::Response<Body>
    where
        Backend: axum_login::AuthnBackend<Error: IntoResponse>
        + RequestAuthenticated
        + AuthorizeBackend
        + AuthnBackendAttributes
        + 'static,
        Backend: axum_login::AuthnBackend<User = <Backend as PermissionProviderSource>::User>,
        <Backend as PermissionProviderSource>::Permission: Display,
        <Backend as PermissionProviderSource>::PermissionSet: Display,
{
    validate_authorization_chain(auth_session, required_permissions.0, req, next).await
}
