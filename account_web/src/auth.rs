use mvv_auth::route::validate_authentication_chain;

mod auth_layer;
mod backend;
mod oauth;
mod user;
mod user_perm_provider;
mod login_form;
// -------------------------------------------------------------------------------------------------


pub type AuthUser = ClientAuthUser; // really client in scope of this web-app
pub type Role = ClientType;         // really client features
pub type RolePermissionsSet = user::RolePermissionsSet;
pub type ClientFeatureSetSet = user::RolePermissionsSet;

pub type PswComparator = user_perm_provider::PswComparator;
pub type AuthBackendError = mvv_auth::AuthBackendError;

pub type CompositeAuthCredentials = backend::CompositeAuthCredentials;
pub type CompositeAuthBackend = backend::CompositeAuthBackend;


#[allow(unused_qualifications)] // false-positive warning
pub type PswAuthCredentials = mvv_auth::backend::PswAuthCredentials;
pub type LoginFormAuthnBackend = mvv_auth::backend::LoginFormAuthBackend<AuthUser,PswComparator,RolePermissionsSet>;


pub use user_perm_provider::{ ClientAuthUserProvider, client_auth_user_provider };
pub use auth_layer::{ composite_auth_manager_layer, /*login_form_auth_manager_layer*/ };
pub use login_form::{ composite_login_router };
pub use user::{ ClientType, ClientAuthUser };

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
    fn client_type_required(self, client_type: ClientType) -> Self {
        use mvv_auth::permission::PermissionSet;
        self.route_layer(axum::middleware::from_fn_with_state(
            RolePermissionsSet::from_permission(client_type),
            mvv_auth::route::validate_authorization_chain_as_middleware_fn::<AuthBackend>
        ))
    }
    #[track_caller]
    fn client_features_required(self, client_features: ClientFeatureSetSet) -> Self {
        self.route_layer(axum::middleware::from_fn_with_state(
            client_features,
            mvv_auth::route::validate_authorization_chain_as_middleware_fn::<AuthBackend>,
        ))
    }
}
