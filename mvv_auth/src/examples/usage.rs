use axum::extract::{ Request, State };
use axum::body::Body;

use crate::{
    AuthBackendError,
};
use crate::examples::auth_user::AuthUserExample;
use crate::route::validate_authentication_chain;
use super::composite_auth::{
    CompositeAuthCredentials, CompositeAuthnBackendExample, Role, RolePermissionsSet,
};


#[extension_trait::extension_trait]
pub impl <S: Clone + Send + Sync + 'static> RequiredAuthenticationExtension for axum::Router<S> {
    // #[inline] // warning: `#[inline]` is ignored on function prototypes
    #[track_caller]
    fn authn_required(self) -> Self {
        self.route_layer(axum::middleware::from_fn(
            validate_authentication_chain::
                <AuthUserExample, CompositeAuthCredentials, AuthBackendError, CompositeAuthnBackendExample>))
    }
}


#[inline]
pub async fn validate_authorization_chain_as_middleware_fn<
> (
    auth_session: axum_login::AuthSession<CompositeAuthnBackendExample>,
    required_permissions: State<RolePermissionsSet>,
    req: Request, next: axum::middleware::Next,
) -> http::Response<Body> {
    crate::route::validate_authorization_chain_as_middleware_fn(
        auth_session, required_permissions, req, next,
    ).await
}


#[extension_trait::extension_trait]
pub impl <S: Clone + Send + Sync + 'static> RequiredAuthorizationExtension for axum::Router<S> {
    #[track_caller]
    fn role_required(self, role: Role) -> Self {
        use crate::permission::PermissionSet;
        self.route_layer(axum::middleware::from_fn_with_state(
            RolePermissionsSet::from_permission(role),
            validate_authorization_chain_as_middleware_fn))
    }
    #[track_caller]
    fn roles_required(self, roles: RolePermissionsSet) -> Self {
        self.route_layer(axum::middleware::from_fn_with_state(
            roles, validate_authorization_chain_as_middleware_fn))
    }
}
