use axum::extract::{ Request, State };
use axum::body::Body;
use axum::response::IntoResponse;
use http::status::StatusCode;
use log::error;
use crate::auth::AuthnBackendAttributes;

use crate::auth::backend::authz_backend::AuthorizeBackend;
use crate::auth::backend::RequestAuthenticated;

use crate::auth::examples::composite_auth::{ CompositeAuthnBackendExample, Role, RolePermissionsSet };
use crate::auth::permission::util::log_unauthorized_access;


pub async fn validate_authentication_chain (
    auth_session: axum_login::AuthSession<CompositeAuthnBackendExample>,
    req: Request,
    next: axum::middleware::Next,
) -> http::Response<Body> {

    use super::super::backend::RequestAuthenticated;

    let (req, is_auth_res) =
        auth_session.backend.do_authenticate_request::<()>(req).await;
    match is_auth_res {
        Ok(None) => StatusCode::UNAUTHORIZED.into_response(),
        Ok(_) => next.run(req).await,
        Err(action) => action.into_response()
    }
}


#[extension_trait::extension_trait]
pub impl <S: Clone + Send + Sync + 'static> RequiredAuthenticationExtension for axum::Router<S> {
    // #[inline] // warning: `#[inline]` is ignored on function prototypes
    #[track_caller]
    fn authn_required(self) -> Self {
        self.route_layer(axum::middleware::from_fn(validate_authentication_chain))
    }
}


/*
async fn do_is_authorized (
    req: Request,
    auth_session: axum_login::AuthSession<CompositeAuthnBackendExample>,
    required_permissions: RolePermissionsSet,
) -> (Request,Result<bool, PermissionProcessError>) {

    if let Some(ref user) = auth_session.user {
        let authz_res = auth_session.backend.authorize(user, required_permissions).await;
        match authz_res {
            Ok(res) => {
                let (req,) = log_unauthorized_access(req, user, &res);
                (req, Ok(res.is_authorized()))
            }
            Err(err) => (req, Err(err)),
        }
    } else {
        (req, Ok(false))
    }
}


pub async fn validate_authorization_chain (
    auth_session: axum_login::AuthSession<CompositeAuthnBackendExample>,
    required_permissions: RolePermissionsSet,
    req: Request,
    next: axum::middleware::Next,
) -> http::Response<Body> {
    let (req, is_auth_res) = do_is_authorized(req, auth_session, required_permissions).await;
    match is_auth_res {
        Ok(true) =>
            next.run(req).await,
        Ok(false) =>
            // Probably by security reason it would be better return 401/404. It is up to you.
            StatusCode::FORBIDDEN.into_response(),
        Err(action) =>
            action.into_response(),
    }
}
*/


pub async fn validate_authorization_chain (
    auth_session: axum_login::AuthSession<CompositeAuthnBackendExample>,
    required_permissions: RolePermissionsSet,
    req: Request,
    next: axum::middleware::Next,
) -> http::Response<Body> {

    let (req, user_opt_res) = auth_session.backend
        .do_authenticate_request::<()>(req).await; // , auth_session.clone()).await;

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
async fn internal_validate_authorization_chain (
    auth_session: axum_login::AuthSession<CompositeAuthnBackendExample>,
    State(required_permissions): State<RolePermissionsSet>,
    req: Request, next: axum::middleware::Next,
) -> http::Response<Body> {
    validate_authorization_chain(auth_session, required_permissions, req, next).await
}


#[extension_trait::extension_trait]
pub impl <S: Clone + Send + Sync + 'static> RequiredAuthorizationExtension for axum::Router<S> {
    #[track_caller]
    fn role_required(self, role: Role) -> Self {
        use crate::auth::permission::PermissionSet;
        self.route_layer(axum::middleware::from_fn_with_state(
            RolePermissionsSet::from_permission(role),
            internal_validate_authorization_chain))
    }
    #[track_caller]
    fn roles_required(self, roles: RolePermissionsSet) -> Self {
        self.route_layer(axum::middleware::from_fn_with_state(roles, internal_validate_authorization_chain))
    }
}
