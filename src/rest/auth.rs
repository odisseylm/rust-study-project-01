use mvv_auth::{
    backend::LoginFormAuthBackend,
    route::validate_authentication_chain,
};


// -------------------------------------------------------------------------------------------------
//                                   Include sub-modules
// -------------------------------------------------------------------------------------------------
pub mod backend;
pub mod user;
pub mod user_perm_provider;
pub mod auth_layer;
// -------------------------------------------------------------------------------------------------


pub type AuthUser = user::AuthUser;
pub type Role = mvv_auth::permission::predefined::Role;
pub type RolePermissionsSet = mvv_auth::permission::predefined::RolePermissionsSet;

pub type PswComparator = user_perm_provider::PswComparator;
pub type AuthBackendError = mvv_auth::AuthBackendError;

pub type CompositeAuthCredentials = backend::CompositeAuthCredentials;
pub type CompositeAuthBackend = backend::CompositeAuthBackend;


#[allow(unused_qualifications)] // false-positive warning
pub type PswAuthCredentials = mvv_auth::backend::PswAuthCredentials;
pub type LoginFormAuthnBackend = LoginFormAuthBackend<AuthUser,PswComparator,RolePermissionsSet>;


// -------------------------------------------------------------------------
//                           Currently used impls
// -------------------------------------------------------------------------
pub type AuthCredentials = CompositeAuthCredentials;
pub type AuthBackend = CompositeAuthBackend;
// pub type AuthCredentials = PswAuthCredentials;
// pub type AuthnBackend = LoginFormAuthnBackend;
pub type AuthSession = axum_login::AuthSession<AuthBackend>;


#[extension_trait::extension_trait]
pub impl <S: Clone + Send + Sync + 'static> RequiredAuthenticationExtension for axum::Router<S> {
    #[track_caller]
    fn authn_required(self) -> Self {
        self.route_layer(axum::middleware::from_fn(
            validate_authentication_chain::<AuthBackend>))
    }
}


#[extension_trait::extension_trait]
pub impl <S: Clone + Send + Sync + 'static> RequiredAuthorizationExtension for axum::Router<S> {
    #[track_caller]
    fn role_required(self, role: Role) -> Self {
        use mvv_auth::permission::PermissionSet;
        self.route_layer(axum::middleware::from_fn_with_state(
            RolePermissionsSet::from_permission(role),
            mvv_auth::route::validate_authorization_chain_as_middleware_fn::<AuthBackend>
        ))
    }
    #[track_caller]
    fn roles_required(self, roles: RolePermissionsSet) -> Self {
        self.route_layer(axum::middleware::from_fn_with_state(
            roles,
            mvv_auth::route::validate_authorization_chain_as_middleware_fn::<AuthBackend>,
        ))
    }
}

