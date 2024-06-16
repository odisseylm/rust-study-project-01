use std::sync::Arc;
use axum::body::Body;
use axum::extract::Request;
use axum::response::IntoResponse;
use oauth2::basic::BasicClient;
use oauth2_auth::OAuth2AuthBackend;
use crate::auth::{HttpBasicAuthMode, InMemAuthUserProvider, LoginFormAuthMode, PlainPasswordComparator, UnauthenticatedAction};
use crate::auth::composite_auth::CompositeAuthnBackend;
use crate::auth::oauth2_auth::Oauth2Config;
use crate::auth::oauth2_auth;


pub type AuthUser = crate::auth::AuthUser;
pub type AuthCredentials = crate::auth::composite_auth::CompositeAuthCredentials;
pub type AuthnBackend = crate::auth::composite_auth::CompositeAuthnBackend;
pub type AuthSession = axum_login::AuthSession<CompositeAuthnBackend>;
pub type AuthBackendError = crate::auth::AuthBackendError;


async fn is_authenticated (
    auth_session: AuthSession,
    req: Request,
) -> (Request, Result<(), UnauthenticatedAction>) {
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

    // Auth service.
    //
    // This combines the session layer with our backend to establish the auth
    // service which will provide the auth session as a request extension.

    let usr_provider_impl: Arc<InMemAuthUserProvider> = Arc::new(InMemAuthUserProvider::test_users() ?);

    // Rust does not support casting dyn sub-trait to dyn super-trait :-(
    // let std_usr_provider: Arc<dyn crate::auth::AuthUserProvider<User = AuthUser> + Send + Sync> = wrap_static_ptr_auth_user_provider(Arc::clone(&usr_provider_impl));
    // Seems we may not use wrap_static_ptr_auth_user_provider(Arc::clone(&usr_provider_impl))
    // but we need to use usr_provider_impl.clone(), NOT Arc::clone(&usr_provider_impl) !!!
    // !!! With Arc::clone(&usr_provider_impl) auto casting does NOT work !!!
    //
    let std_usr_provider: Arc<dyn crate::auth::AuthUserProvider<User = AuthUser> + Send + Sync> = usr_provider_impl.clone();
    let oauth2_usr_store: Arc<dyn crate::auth::OAuth2UserStore<User = AuthUser> + Sync + Send> = usr_provider_impl; // or .clone();

    let config = Oauth2Config::git_from_env() ?;
    let oauth2_backend_opt: Option<OAuth2AuthBackend> = match config {
        None => None,
        Some(config) => {
            let oauth2_basic_client: BasicClient = oauth2_auth::create_basic_client(&config) ?;
            Some(OAuth2AuthBackend::new(
                Arc::clone(&oauth2_usr_store),
                LoginFormAuthMode::LoginFormAuthProposed { login_form_url: Some("/login") },
                oauth2_basic_client,
            ))
        }
    };

    let http_basic_auth_backend = crate::auth::HttpBasicAuthBackend::<PlainPasswordComparator>::new(
        std_usr_provider.clone(),
        // It makes sense for pure server SOA
        // BasicAuthMode::BasicAuthProposed,
        HttpBasicAuthMode::BasicAuthSupported,
    );
    let login_form_auth_backend = crate::auth::LoginFormAuthBackend::<PlainPasswordComparator>::new(
        std_usr_provider.clone(),
        // It makes sense for web-app
        LoginFormAuthMode::LoginFormAuthProposed { login_form_url: Some("/login") }
    );

    let backend = AuthnBackend::new_raw(
        std_usr_provider.clone(),
        Some(http_basic_auth_backend), Some(login_form_auth_backend), oauth2_backend_opt);
    let auth_layer: axum_login::AuthManagerLayer<AuthnBackend, MemoryStore> = AuthManagerLayerBuilder::new(backend, session_layer).build();
    Ok(auth_layer)
}
