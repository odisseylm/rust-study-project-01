use std::sync::Arc;
use axum_login::UserId;
use oauth2::basic::BasicClient;
use super::psw_auth;
use crate::auth::auth_user;
use crate::auth::auth_user::AuthUserProviderError;
use crate::auth::oauth2_auth;
use crate::auth::psw::PlainPasswordComparator;
use crate::auth::mem_user_provider::InMemAuthUserProvider;

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


// #[derive(Clone)]
pub struct AuthnBackend <
    // UsrProvider: AuthUserProvider<User = auth_user::AuthUser> + Sync + Send, // + Clone + Sync + Send,
    > {
    psw_backend: Option<psw_auth::AuthBackend<PlainPasswordComparator>>,
    oauth2_backend: Option<oauth2_auth::Backend>,
}

impl AuthnBackend {
    async fn test_users() -> Result<AuthnBackend, anyhow::Error> {
        /*
        use oauth2::{ ClientId, ClientSecret, AuthUrl, TokenUrl };

        let client_id = std::env::var("CLIENT_ID")
            .map(ClientId::new)
            .expect("CLIENT_ID should be provided.");
        let client_secret = std::env::var("CLIENT_SECRET")
            .map(ClientSecret::new)
            .expect("CLIENT_SECRET should be provided");

        let auth_url = AuthUrl::new("https://github.com/login/oauth/authorize".to_string())?;
        let token_url = TokenUrl::new("https://github.com/login/oauth/access_token".to_string())?;
        let basic_client = BasicClient::new(client_id, Some(client_secret), auth_url, Some(token_url));

        // let db = SqlitePool::connect(":memory:").await?;
        // sqlx::migrate!().run(&db).await?;
        */

        Ok(AuthnBackend {
            psw_backend: Some(
                psw_auth::AuthBackend::new(
                    Arc::new(InMemAuthUserProvider::new()))),
            oauth2_backend: None,
            // oauth2_backend: Some(
            //     oauth2_auth::Backend::new(todo!(), basic_client)),
        })
    }
}


impl AuthnBackend {

    pub fn oath2_only(_client: BasicClient) -> AuthnBackend {
        todo!()
    }

    pub fn authorize_url(&self) -> Result<(oauth2::url::Url, oauth2::CsrfToken), AuthError> {
        match self.oauth2_backend {
            None => Err(AuthError::NoRequestedBackend),
            Some(ref oauth2_backend) => Ok(oauth2_backend.authorize_url()),
        }
    }
}


impl Clone for AuthnBackend {
    fn clone(&self) -> Self {
        AuthnBackend {
            psw_backend: self.psw_backend.clone(),
            oauth2_backend: self.oauth2_backend.clone(),
        }
    }
    fn clone_from(&mut self, source: &Self) {
        self.psw_backend = source.psw_backend.clone();
    }
}

#[axum::async_trait]
impl axum_login::AuthnBackend for AuthnBackend {
    type User = auth_user::AuthUser;
    type Credentials = AuthCredentials;
    type Error = AuthError;

    async fn authenticate(&self, creds: Self::Credentials) -> Result<Option<Self::User>, Self::Error> {
        match creds {
            AuthCredentials::Password(creds) =>
                match self.psw_backend {
                    None => Err(AuthError::NoRequestedBackend),
                    Some(ref backend) => backend.authenticate(creds).await.map_err(AuthError::from)
                },
            AuthCredentials::OAuth(creds) =>
                match self.oauth2_backend {
                    None => Err(AuthError::NoRequestedBackend),
                    Some(ref backend) => backend.authenticate(creds).await.map_err(AuthError::from)
                },
        }
    }

    async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        // expected that app uses only one Users Provider (in all auth backends)
        let res = match self.psw_backend {
            None => Err(AuthError::NoRequestedBackend),
            Some(ref backend) => backend.get_user(user_id).await.map_err(AuthError::from),
        };

        if res.is_ok() { return res }

        // TODO: simplify
        let res = match self.oauth2_backend {
            None => Err(AuthError::NoRequestedBackend),
            Some(ref backend) => backend.get_user(user_id).await.map_err(AuthError::from),
        };

        res
    }
}

pub type AuthSession = axum_login::AuthSession<AuthnBackend>;


pub type OAuthCreds = oauth2_auth::Credentials;
pub type PasswordCreds = psw_auth::AuthCredentials;

#[derive(Debug, Clone, serde::Deserialize)]
pub enum AuthCredentials {
    Password(PasswordCreds),
    OAuth(OAuthCreds),
}


#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("NoUser")]
    NoUser,

    #[error("NoRequestedBackend")]
    NoRequestedBackend,

    #[error("IncorrectUsernameOrPsw")]
    IncorrectUsernameOrPsw,

    #[error("UserProviderError")]
    UserProviderError(AuthUserProviderError),

    #[error(transparent)]
    OAuth2(oauth2::basic::BasicRequestTokenError<oauth2::reqwest::AsyncHttpClientError>),

    #[error(transparent)]
    Sqlx(sqlx::Error),

    #[error(transparent)]
    Reqwest(reqwest::Error),

    #[error(transparent)]
    TaskJoin(#[from] tokio::task::JoinError),
}

impl From<psw_auth::AuthError> for AuthError {
    fn from(value: psw_auth::AuthError) -> Self {
        use psw_auth as psw;
        match value {
            psw::AuthError::NoUser => AuthError::NoUser,
            psw::AuthError::IncorrectUsernameOrPsw => AuthError::IncorrectUsernameOrPsw,
            psw::AuthError::UserProviderError(err) => AuthError::UserProviderError(err),
        }
    }
}


impl From<AuthUserProviderError> for AuthError {
    fn from(value: AuthUserProviderError) -> Self {
        AuthError::UserProviderError(value)
    }
}


impl From<oauth2_auth::BackendError> for AuthError {
    fn from(value: oauth2_auth::BackendError) -> Self {
        use oauth2_auth as o;
        match value {
            o::BackendError::Reqwest(cause) => AuthError::Reqwest(cause),
            o::BackendError::OAuth2(cause) => AuthError::OAuth2(cause),
            o::BackendError::UserProviderError(err) => AuthError::UserProviderError(err),
            o::BackendError::Sqlx(err) => AuthError::Sqlx(err),
        }
    }
}
