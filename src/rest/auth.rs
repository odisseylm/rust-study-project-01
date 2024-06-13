use std::sync::Arc;
use axum::body::Body;
use axum_extra::TypedHeader;
use axum_extra::headers::Authorization as AuthorizationHeader;
use axum_extra::headers::authorization::Basic;
use oauth2::basic::BasicClient;
use crate::auth::{ InMemAuthUserProvider, PlainPasswordComparator, wrap_static_ptr_auth_user_provider };
use crate::auth::oauth2_auth::Oauth2Config;
use crate::auth::oauth2_auth;


pub type AuthUser = crate::auth::AuthUser;
pub type AuthCredentials = crate::auth::composite_auth::AuthCredentials;
pub type AuthnBackend = crate::auth::composite_auth::AuthnBackend;
pub type AuthSession = crate::auth::composite_auth::AuthSession;


async fn is_authenticated (
    auth_session: AuthSession,
    basic_auth_creds: Option<TypedHeader<AuthorizationHeader<Basic>>>,
) -> bool {
    crate::auth::composite_auth::is_authenticated(auth_session, basic_auth_creds).await
}


#[inline]
pub async fn validate_auth_temp(
    auth_session: AuthSession, basic_auth_creds: Option<TypedHeader<AuthorizationHeader<Basic>>>,
    req: axum::extract::Request, next: axum::middleware::Next) -> http::Response<Body> {
    validate_auth(auth_session, basic_auth_creds, req, next).await
}

pub async fn validate_auth(
    auth_session: AuthSession,
    basic_auth_creds: Option<TypedHeader<AuthorizationHeader<Basic>>>,
    req: axum::extract::Request,
    next: axum::middleware::Next
) -> http::Response<Body> {
    if is_authenticated(auth_session, basic_auth_creds).await {
        next.run(req).await
    } else {
        // or redirect to login page
        // should be configurable outside: dev or prod
        super::error_rest::unauthenticated_401_response()
    }
}


#[extension_trait::extension_trait]
pub impl<S: Clone + Send + Sync + 'static> RequiredAuthenticationExtension for axum::Router<S> {
    // #[inline] // warning: `#[inline]` is ignored on function prototypes
    #[track_caller]
    fn auth_required(self) -> Self {
        self.route_layer(axum::middleware::from_fn(validate_auth))
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

    // Auth service.
    //
    // This combines the session layer with our backend to establish the auth
    // service which will provide the auth session as a request extension.

    let usr_provider_impl: Arc<InMemAuthUserProvider> = Arc::new(InMemAuthUserProvider::test_users().await ?);
    // Rust does not support casting dyn sub-trait to dyn super-trait :-(
    let std_usr_provider: Arc<dyn crate::auth::AuthUserProvider<User = AuthUser> + Send + Sync> = wrap_static_ptr_auth_user_provider(Arc::clone(&usr_provider_impl));
    let oauth2_usr_store: Arc<dyn crate::auth::Oauth2UserStore<User = AuthUser> + Sync + Send> = usr_provider_impl;

    let config = Oauth2Config::git_from_env() ?;
    let oauth2_backend: Option<oauth2_auth::AuthBackend> = match config {
        None => None,
        Some(config) => {
            let oauth2_basic_client: BasicClient = oauth2_auth::create_basic_client(&config) ?;
            Some(oauth2_auth::AuthBackend::new(Arc::clone(&oauth2_usr_store), oauth2_basic_client))
        }
    };

    let psw_auth_backend = crate::auth::PswAuthBackend::<PlainPasswordComparator>::new(std_usr_provider);

    let backend = AuthnBackend::new_raw(Some(psw_auth_backend), oauth2_backend);
    let auth_layer: axum_login::AuthManagerLayer<AuthnBackend, MemoryStore> = AuthManagerLayerBuilder::new(backend, session_layer).build();
    Ok(auth_layer)
}
