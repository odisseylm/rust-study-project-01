use std::sync::Arc;
use axum_login::UserId;
// use axum_login::UserId;
use oauth2::basic::BasicClient;
// use crate::auth::auth_user_provider as auth;
use super::by_psw_auth;
use crate::auth::auth_user::{ self, AuthUserProvider };
use crate::auth::psw::PlainPasswordComparator;
use crate::auth::authn_backend_dyn_wrapper::AuthnBackendDynWrapper;
use crate::auth::by_psw_auth::TestAuthUserProvider;
use super::authn_backend_dyn_wrapper::wrap_authn_backend_as_dyn;

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
    UsrProvider: AuthUserProvider<User = auth_user::AuthUser> + Sync + Send, // + Clone + Sync + Send,
    > {
    psw_backend: Arc<dyn AuthnBackendDynWrapper<
        Credentials = by_psw_auth::AuthCredentials,
        Error = by_psw_auth::AuthError,
        RealAuthnBackend = by_psw_auth::AuthBackend<UsrProvider, PlainPasswordComparator>
    >>,
}

impl AuthnBackend<TestAuthUserProvider> {
    fn test_users() -> AuthnBackend<TestAuthUserProvider> {
        AuthnBackend {
            psw_backend: Arc::new(
                wrap_authn_backend_as_dyn(
                    by_psw_auth::AuthBackend::new(
                        Arc::new(TestAuthUserProvider::new())))),
        }
    }
}


impl <
    UsrProvider: AuthUserProvider<User = auth_user::AuthUser> + Sync + Send, // + Clone + Sync + Send,
> AuthnBackend<UsrProvider> {
    // fn test_users() -> AuthnBackend<UsrProvider> {
    //     todo!()
    // }

    pub fn oath2_only(_client: BasicClient) -> AuthnBackend<UsrProvider> {
        todo!()
    }

    pub fn authorize_url(&self) -> (oauth2::url::Url, oauth2::CsrfToken) {
        //
        // self.client.authorize_url(oauth2::CsrfToken::new_random).url()
        todo!() // TODO: redirect to OAuth2Backend
    }
}


impl <
    UsrProvider: AuthUserProvider<User = auth_user::AuthUser> + Sync + Send, // + Clone + Sync + Send,
> Clone for AuthnBackend<UsrProvider> {
    fn clone(&self) -> Self {
        AuthnBackend::<UsrProvider> {
            psw_backend: self.psw_backend.clone(),
        }
    }
    fn clone_from(&mut self, source: &Self) {
        self.psw_backend = source.psw_backend.clone();
    }
}

#[axum::async_trait]
impl <
    UsrProvider: AuthUserProvider<User = auth_user::AuthUser> + Sync + Send, // + Clone + Sync + Send,
> axum_login::AuthnBackend for AuthnBackend<UsrProvider> {
    type User = auth_user::AuthUser;
    type Credentials = AuthCredentials;
    type Error = AuthError;

    async fn authenticate(&self, creds: Self::Credentials) -> Result<Option<Self::User>, Self::Error> {
        match creds {
            AuthCredentials::Password(psw_creds) => {
                self.psw_backend.authenticate(psw_creds).await.map_err(AuthError::from)
                // let aa = self.psw_backend.authenticate(psw_creds).await ?;
                // Ok(aa)
            }
            AuthCredentials::OAuth(_) => { todo!() }
        }
    }

    async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        self.psw_backend.get_user(user_id).await.map_err(AuthError::from)
    }
}

// pub type AuthSession<UsrProvider: AuthUserProvider + Send + Sync> = axum_login::AuthSession<AuthnBackend<UsrProvider>>;
pub type AuthSession<UsrProvider> = axum_login::AuthSession<AuthnBackend<UsrProvider>>;


pub type OAuthCreds = super::oauth2_auth::Credentials;
pub type PasswordCreds = super::by_psw_auth::AuthCredentials;

#[derive(Debug, Clone, serde::Deserialize)]
pub enum AuthCredentials {
    Password(PasswordCreds),
    OAuth(OAuthCreds),
}


#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("NoUser")]
    NoUser,

    #[error("IncorrectUsernameOrPsw")]
    IncorrectUsernameOrPsw,

    #[error("UserProviderError")]
    UserProviderError,

    #[error(transparent)]
    OAuth2(oauth2::basic::BasicRequestTokenError<oauth2::reqwest::AsyncHttpClientError>),

    #[error(transparent)]
    Sqlx(sqlx::Error),

    // #[error(transparent)]
    // Reqwest(reqwest::Error),

    // #[error(transparent)]
    // TaskJoin(#[from] task::JoinError),
}

impl From<by_psw_auth::AuthError> for AuthError {
    fn from(value: by_psw_auth::AuthError) -> Self {
        use by_psw_auth as psw;
        match value {
            psw::AuthError::NoUser => AuthError::NoUser,
            psw::AuthError::IncorrectUsernameOrPsw => AuthError::IncorrectUsernameOrPsw,
            psw::AuthError::UserProviderError => AuthError::UserProviderError,
        }
    }
}

