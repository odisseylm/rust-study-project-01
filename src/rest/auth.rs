use std::sync::Arc;
use axum::body::Body;
use axum::extract::Request;
use mvv_auth::{ AuthBackendMode, AuthUserProviderError, PlainPasswordComparator };
use mvv_auth::user_provider::{ InMemAuthUserProvider };
use mvv_auth::backend::{ LoginFormAuthConfig, OAuth2AuthBackend, OAuth2Config };
use mvv_auth::examples::auth_user::{ AuthUserExamplePswExtractor };
use mvv_auth::permission::{ PermissionProvider };


pub type AuthUser = mvv_auth::examples::auth_user::AuthUserExample;
pub type AuthCredentials = mvv_auth::examples::composite_auth::CompositeAuthCredentials;
pub type AuthnBackend = mvv_auth::examples::composite_auth::CompositeAuthnBackendExample;
pub type AuthSession = axum_login::AuthSession<AuthnBackend>;
pub type AuthBackendError = mvv_auth::AuthBackendError;
pub type Role = mvv_auth::permission::predefined::Role;
pub type RolePermissionsSet = mvv_auth::permission::predefined::RolePermissionsSet;


// Type alias cannot be applied for trait ?! :-(
// pub type RequiredAuthenticationExtension = crate::auth::examples::usage::RequiredAuthenticationExtension;
//
#[extension_trait::extension_trait]
pub impl <S: Clone + Send + Sync + 'static> RequiredAuthenticationExtension for axum::Router<S> {
    #[track_caller]
    fn authn_required(self) -> Self {
        self.route_layer(axum::middleware::from_fn(
            mvv_auth::examples::usage::validate_authentication_chain))
    }
}


// Type alias cannot be applied for trait ?! :-(
// pub type RequiredAuthorizationExtension = crate::auth::examples::usage::RequiredAuthorizationExtension;
//
#[extension_trait::extension_trait]
pub impl <S: Clone + Send + Sync + 'static> RequiredAuthorizationExtension for axum::Router<S> {
    #[track_caller]
    fn role_required(self, role: Role) -> Self {
        use mvv_auth::permission::PermissionSet;
        self.route_layer(axum::middleware::from_fn_with_state(
            RolePermissionsSet::from_permission(role),
            mvv_auth::examples::usage::internal_validate_authorization_chain,
        ))
    }
    #[track_caller]
    fn roles_required(self, roles: RolePermissionsSet) -> Self {
        self.route_layer(axum::middleware::from_fn_with_state(
            roles,
            mvv_auth::examples::usage::internal_validate_authorization_chain,
        ))
    }
}


#[inline]
pub async fn validate_auth_temp(
    auth_session: AuthSession,
    req: Request,
    next: axum::middleware::Next,
) -> http::Response<Body> {
    mvv_auth::examples::usage::validate_authentication_chain(auth_session, req, next).await
}


pub async fn auth_manager_layer() -> Result<axum_login::AuthManagerLayer<AuthnBackend, axum_login::tower_sessions::MemoryStore>, anyhow::Error> {

    use axum_login::{
        // login_required,
        tower_sessions::{cookie::SameSite, Expiry, MemoryStore, SessionManagerLayer},
        AuthManagerLayerBuilder,
    };
    use time::Duration;

    // This uses `tower-sessions` to establish a layer that will provide the session
    // as a request extension.
    //
    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_same_site(SameSite::Lax) // Ensure we send the cookie from the OAuth redirect.
        .with_expiry(Expiry::OnInactivity(Duration::days(1)));

    // This combines the session layer with our backend to establish the auth service
    // which will provide the auth session as a request extension.
    //
    // let usr_provider: Arc<InMemAuthUserProvider<AuthUser>> = Arc::new(InMemAuthUserProvider::test_users() ?);
    let usr_provider: Arc<InMemAuthUserProvider<AuthUser,Role,RolePermissionsSet,AuthUserExamplePswExtractor>> =
        Arc::new(in_memory_test_users() ?);
    let permission_provider: Arc<dyn PermissionProvider<User=AuthUser,Permission=Role,PermissionSet=RolePermissionsSet>> = usr_provider.clone();

    // Rust does not support casting dyn sub-trait to dyn super-trait :-(
    // let std_usr_provider: Arc<dyn crate::auth::AuthUserProvider<User = AuthUser> + Send + Sync> = wrap_static_ptr_auth_user_provider(Arc::clone(&usr_provider_impl));
    // Seems we may not use wrap_static_ptr_auth_user_provider(Arc::clone(&usr_provider_impl))
    // but we need to use usr_provider_impl.clone(), NOT Arc::clone(&usr_provider_impl) !!!
    // !!! With Arc::clone(&usr_provider_impl) auto casting does NOT work !!!
    //
    let config = OAuth2Config::git_from_env() ?;
    let oauth2_backend_opt: Option<OAuth2AuthBackend<AuthUser,Role,RolePermissionsSet>> = match config {
        None => None,
        Some(config) => {
            let mut config = config.clone();
            // config.auth_mode = AuthBackendMode::AuthProposed;
            config.login_url = "/login";

            Some(OAuth2AuthBackend::new(
                usr_provider.clone(),
                usr_provider.clone(), // it is automatically cast to another 'dyn' object. It should be done THERE!
                config,
                None, // oauth2_basic_client,
                permission_provider.clone(),
            ) ?)
        }
    };

    let http_basic_auth_backend = mvv_auth::backend::HttpBasicAuthBackend::<AuthUser,PlainPasswordComparator,Role,RolePermissionsSet>::new(
        usr_provider.clone(),
        AuthBackendMode::AuthProposed, // It makes sense for pure server SOA (especially for testing)
        // AuthBackendMode::AuthSupported,
        permission_provider.clone(),
    );
    let login_form_auth_backend = mvv_auth::backend::LoginFormAuthBackend::<AuthUser,PlainPasswordComparator,Role,RolePermissionsSet>::new(
        usr_provider.clone(),
        // It makes sense for web-app
        LoginFormAuthConfig {
            auth_mode: AuthBackendMode::AuthProposed,
            login_url: "/login",
        },
        permission_provider.clone(),
    );

    let backend = AuthnBackend::with_backends(
        Some(http_basic_auth_backend),
        Some(login_form_auth_backend),
        // None,
        oauth2_backend_opt,
    ) ?;
    let auth_layer: axum_login::AuthManagerLayer<AuthnBackend, MemoryStore> = AuthManagerLayerBuilder::new(backend, session_layer).build();
    Ok(auth_layer)
}

pub fn in_memory_test_users() -> Result<InMemAuthUserProvider<AuthUser,Role,RolePermissionsSet,AuthUserExamplePswExtractor>, AuthUserProviderError> {
    mvv_auth::examples::composite_auth::test::in_memory_test_users()
}
