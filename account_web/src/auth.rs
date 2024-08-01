use mvv_auth::route::validate_authentication_chain;

mod auth_layer;
mod backend;
mod oauth;
mod user;
mod user_perm_provider;
mod login_form;
mod sql_client_auth_provider;
// -------------------------------------------------------------------------------------------------


pub use user::{ ClientAuthUser, ClientFeature, ClientFeatureSet };
// just aliases for easy copy-paste other authn/authz related code
pub type AuthUser = ClientAuthUser;
pub type Role = ClientFeature;
pub type RolePermissionsSet = ClientFeatureSet;

pub type PswComparator = user_perm_provider::PswComparator;
pub type AuthBackendError = mvv_auth::AuthBackendError;

pub type CompositeAuthCredentials = backend::CompositeAuthCredentials;
pub type CompositeAuthBackend = backend::CompositeAuthBackend;


#[allow(unused_qualifications)] // false-positive warning
pub type PswAuthCredentials = mvv_auth::backend::PswAuthCredentials;
pub type LoginFormAuthnBackend = mvv_auth::backend::LoginFormAuthBackend<AuthUser,PswComparator,RolePermissionsSet>;


pub use user_perm_provider::{ AuthUserProvider, in_mem_client_auth_user_provider };
pub use auth_layer::{ composite_auth_manager_layer, /*login_form_auth_manager_layer*/ };
pub use login_form::{ composite_login_router };
pub use sql_client_auth_provider::SqlClientAuthUserProvider;

// -------------------------------------------------------------------------
//                           Currently used impls
// -------------------------------------------------------------------------
pub type AuthCredentials = CompositeAuthCredentials;
pub type AuthBackend = CompositeAuthBackend;
// pub type AuthCredentials = PswAuthCredentials;
// pub type AuthnBackend = LoginFormAuthnBackend;
pub type AuthSession = axum_login::AuthSession<AuthBackend>;

pub type ExtractCurrentUser = mvv_auth::extract::ExtractCurrentUser<ClientAuthUser, AuthBackend>;



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
    fn client_feature_required(self, client_feature: ClientFeature) -> Self {
        use mvv_auth::permission::PermissionSet;
        self.route_layer(axum::middleware::from_fn_with_state(
            ClientFeatureSet::from_permission(client_feature),
            mvv_auth::route::validate_authorization_chain_as_middleware_fn::<AuthBackend>
        ))
    }
    #[track_caller]
    fn client_features_required(self, client_features: ClientFeatureSet) -> Self {
        self.route_layer(axum::middleware::from_fn_with_state(
            client_features,
            mvv_auth::route::validate_authorization_chain_as_middleware_fn::<AuthBackend>,
        ))
    }
}
