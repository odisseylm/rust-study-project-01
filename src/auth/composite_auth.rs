use std::sync::Arc;
use oauth2::basic::BasicClient;
use crate::auth::auth_user_provider::AuthUser;

/*
async fn is_authenticated(
    auth_session: AuthSession,
    basic_auth_creds: Option<TypedHeader<AuthorizationHeader<Basic>>>,
    ) -> bool
{
    if auth_session.user.is_some() {
        return true;
    }

    if let Some(TypedHeader(AuthorizationHeader(ref creds))) = basic_auth_creds {
        // T O D O: avoid to_string()
        let is_auth_res = auth_session.authenticate(AuthCredentials { username: creds.username().to_string(), password: creds.password().to_string() }).await;
        is_auth_res.is_ok()
    }
    else { false }
}


#[inline]
pub async fn validate_auth_temp(
    auth_session: AuthSession, basic_auth_creds: Option<TypedHeader<AuthorizationHeader<Basic>>>,
    req: axum::extract::Request, next: axum::middleware::Next) -> axum::http::Response<Body> {
    validate_auth(auth_session, basic_auth_creds, req, next).await
}

pub async fn validate_auth(
    auth_session: AuthSession,
    basic_auth_creds: Option<TypedHeader<AuthorizationHeader<Basic>>>,
    req: axum::extract::Request,
    next: axum::middleware::Next
) -> axum::http::Response<Body> {
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

pub fn auth_manager_layer() -> axum_login::AuthManagerLayer<AuthnBackend, axum_login::tower_sessions::MemoryStore> {

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
    let backend = AuthnBackend::new();
    let auth_layer: axum_login::AuthManagerLayer<AuthnBackend, MemoryStore> = AuthManagerLayerBuilder::new(backend, session_layer).build();
    auth_layer
}
*/

trait Trait123: Clone {
    fn do_smth(&self);
}


#[axum::async_trait]
pub trait AuthnBackend22: Clone + Sized /*: Clone + Send + Sync*/ {
    /// Authenticating user type.
    type User: axum_login::AuthUser;

    /// Credential type used for authentication.
    type Credentials: Send + Sync;

    /// An error which can occur during authentication and authorization.
    type Error: std::error::Error + Send + Sync;

    /// Authenticates the given credentials with the backend.
    async fn authenticate(
        &self,
        creds: Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error>;

    // Gets the user by provided ID from the backend.
    // async fn get_user(&self, user_id: &axum_login::UserId<Self>) -> Result<Option<Self::User>, Self::Error>;
}


fn aa() {
    // let psw_backends: Box<dyn Trait123> = todo!();
    // let psw_backends: Box<dyn axum_login::AuthnBackend<
    //     User = super::auth_user_provider::AuthUser, // + Clone + Sync + Send + 'static,
    //     Credentials = super::by_psw_auth::AuthCredentials,
    //     Error = super::by_psw_auth::AuthError> + Send + Sync
    // > = todo!();
    let psw_backends: Arc<dyn
        // axum_login::AuthnBackend<
        AuthnBackend22<
            User = super::auth_user_provider::AuthUser,
            Credentials = super::by_psw_auth::AuthCredentials,
            Error = super::by_psw_auth::AuthError,
    >> = todo!();
}

#[derive(Clone)]
pub struct AuthnBackend {
    // psw_backends: Arc<dyn axum_login::AuthnBackend<
    //     User = super::auth_user_provider::AuthUser,
    //     Credentials = super::by_psw_auth::AuthCredentials,
    //     Error = super::by_psw_auth::AuthError>,
    // >,
}

impl AuthnBackend {
    fn test_users() -> AuthnBackend {
        todo!()
    }

    pub fn oath2_only(_client: BasicClient) -> Self {
        todo!()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    // #[error("NoUser")]
    #[error("NoUser")]
    NoUser,

    #[error(transparent)]
    OAuth2(oauth2::basic::BasicRequestTokenError<oauth2::reqwest::AsyncHttpClientError>),

    #[error(transparent)]
    Sqlx(sqlx::Error),

    // #[error(transparent)]
    // Reqwest(reqwest::Error),

    // #[error(transparent)]
    // TaskJoin(#[from] task::JoinError),

    //#[error("IncorrectUsernameOrPsw")]
    #[error("IncorrectUsernameOrPsw")]
    IncorrectUsernameOrPsw, // TODO: Do we need it?
}

pub type AuthSession = axum_login::AuthSession<AuthnBackend>;


pub type OAuthCreds = super::oauth2_auth::Credentials;
pub type PasswordCreds = super::by_psw_auth::AuthCredentials;

#[derive(Debug, Clone, serde::Deserialize)]
pub enum AuthCredentials {
    Password(PasswordCreds),
    OAuth(OAuthCreds),
}
