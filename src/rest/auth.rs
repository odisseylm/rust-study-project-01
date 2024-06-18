use std::sync::Arc;
use axum::body::Body;
use axum::extract::Request;
use axum::response::{ IntoResponse, Response };
use crate::auth::{AuthBackendMode, AuthUserProviderError, PlainPasswordComparator};
use crate::auth::user_provider::{ InMemAuthUserProvider };
use crate::auth::backend::{ LoginFormAuthConfig, OAuth2AuthBackend, OAuth2Config };


pub type AuthUser = crate::auth::AuthUser;
pub type AuthCredentials = crate::auth::examples::composite_auth::CompositeAuthCredentials;
pub type AuthnBackend = crate::auth::examples::composite_auth::CompositeAuthnBackend;
pub type AuthSession = axum_login::AuthSession<AuthnBackend>;
pub type AuthBackendError = crate::auth::AuthBackendError;


async fn is_authenticated (
    auth_session: AuthSession,
    req: Request,
) -> (Request, Result<(), Response>) {
    auth_session.backend.is_authenticated(&auth_session.user, req).await
}


#[inline]
pub async fn validate_auth_temp(
    auth_session: AuthSession,
    req: Request,
    next: axum::middleware::Next,
) -> http::Response<Body> {
    validate_auth_chain(auth_session, req, next).await
}

pub async fn validate_auth_chain (
    auth_session: AuthSession,
    req: Request,
    next: axum::middleware::Next,
) -> http::Response<Body> {

    let (req, is_auth_res) = is_authenticated(auth_session, req).await;
    match is_auth_res {
        Ok(_) => next.run(req).await,
        Err(action) => action.into_response()
    }
}


#[extension_trait::extension_trait]
pub impl<S: Clone + Send + Sync + 'static> RequiredAuthenticationExtension for axum::Router<S> {
    // #[inline] // warning: `#[inline]` is ignored on function prototypes
    #[track_caller]
    fn auth_required(self) -> Self {
        self.route_layer(axum::middleware::from_fn(validate_auth_chain))
    }
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
    let usr_provider: Arc<InMemAuthUserProvider<AuthUser>> = Arc::new(in_memory_test_users() ?);

    // Rust does not support casting dyn sub-trait to dyn super-trait :-(
    // let std_usr_provider: Arc<dyn crate::auth::AuthUserProvider<User = AuthUser> + Send + Sync> = wrap_static_ptr_auth_user_provider(Arc::clone(&usr_provider_impl));
    // Seems we may not use wrap_static_ptr_auth_user_provider(Arc::clone(&usr_provider_impl))
    // but we need to use usr_provider_impl.clone(), NOT Arc::clone(&usr_provider_impl) !!!
    // !!! With Arc::clone(&usr_provider_impl) auto casting does NOT work !!!
    //
    let config = OAuth2Config::git_from_env() ?;
    let oauth2_backend_opt: Option<OAuth2AuthBackend> = match config {
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
            ) ?)
        }
    };

    let http_basic_auth_backend = crate::auth::backend::HttpBasicAuthBackend::<AuthUser,PlainPasswordComparator>::new(
        usr_provider.clone(),
        // AuthBackendMode::AuthProposed, // It makes sense for pure server SOA (especially for testing)
        AuthBackendMode::AuthSupported,
    );
    let login_form_auth_backend = crate::auth::backend::LoginFormAuthBackend::<AuthUser,PlainPasswordComparator>::new(
        usr_provider.clone(),
        // It makes sense for web-app
        LoginFormAuthConfig {
            auth_mode: AuthBackendMode::AuthProposed,
            login_url: "/login",
        },
    );

    let backend = AuthnBackend::with_backends(
        Some(http_basic_auth_backend), Some(login_form_auth_backend), oauth2_backend_opt) ?;
    let auth_layer: axum_login::AuthManagerLayer<AuthnBackend, MemoryStore> = AuthManagerLayerBuilder::new(backend, session_layer).build();
    Ok(auth_layer)
}

pub fn in_memory_test_users() -> Result<InMemAuthUserProvider<AuthUser>, AuthUserProviderError> {
    InMemAuthUserProvider::with_users(vec!(AuthUser::new(1, "vovan", "qwerty")))
}
