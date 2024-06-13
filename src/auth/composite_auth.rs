use std::sync::Arc;
use axum_extra::headers::authorization::Basic;
use axum_extra::headers::Authorization as AuthorizationHeader;
use axum_extra::TypedHeader;

use axum_login::UserId;
use oauth2::basic::BasicClient;

use super::psw_auth;
use super::auth_user;
use super::error::AuthBackendError;
use super::oauth2_auth;
use super::psw::PlainPasswordComparator;
use super::mem_user_provider::InMemAuthUserProvider;


// TODO: configure it in some way??
static USE_HTTP_BASIC_AUTH: bool = true;

pub async fn is_authenticated (
    auth_session: AuthSession,
    basic_auth_creds: Option<TypedHeader<AuthorizationHeader<Basic>>>,
) -> bool {
    if auth_session.user.is_some() {
        return true;
    }

    if USE_HTTP_BASIC_AUTH {
        use crate::auth::psw_auth::AuthCredentials as PswAuthCredentials;

        if let Some(TypedHeader(AuthorizationHeader(ref creds))) = basic_auth_creds {
            let creds = PswAuthCredentials { username: creds.username().to_string(), password: creds.password().to_string(), next: None };
            let is_auth_res = auth_session.authenticate(AuthCredentials::Password(creds)).await;
            return is_auth_res.is_ok()
        }
    };

    return false;
}


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
    > {
    psw_backend: Option<psw_auth::AuthBackend<PlainPasswordComparator>>,
    oauth2_backend: Option<oauth2_auth::AuthBackend>,
}

impl AuthnBackend {
    pub async fn test_users() -> Result<AuthnBackend, anyhow::Error> { // TODO: try to remove async from there
        Ok(AuthnBackend {
            psw_backend: Some(
                psw_auth::AuthBackend::new(
                    Arc::new(InMemAuthUserProvider::new()))),
            oauth2_backend: None,
        })
    }
    pub fn new_raw(
        psw_backend: Option<psw_auth::AuthBackend<PlainPasswordComparator>>,
        oauth2_backend: Option<oauth2_auth::AuthBackend>,
    ) -> AuthnBackend {
        AuthnBackend { psw_backend, oauth2_backend }
    }
}


impl AuthnBackend {

    pub fn oath2_only(_client: BasicClient) -> AuthnBackend {
        todo!()
    }

    pub fn authorize_url(&self) -> Result<(oauth2::url::Url, oauth2::CsrfToken), AuthBackendError> {
        match self.oauth2_backend {
            None => Err(AuthBackendError::NoRequestedBackend),
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
    type Error = AuthBackendError;

    async fn authenticate(&self, creds: Self::Credentials) -> Result<Option<Self::User>, Self::Error> {
        match creds {
            AuthCredentials::Password(creds) =>
                match self.psw_backend {
                    None => Err(AuthBackendError::NoRequestedBackend),
                    Some(ref backend) => backend.authenticate(creds).await.map_err(AuthBackendError::from)
                },
            AuthCredentials::OAuth(creds) =>
                match self.oauth2_backend {
                    None => Err(AuthBackendError::NoRequestedBackend),
                    Some(ref backend) => backend.authenticate(creds).await.map_err(AuthBackendError::from)
                },
        }
    }

    async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        // expected that app uses only one Users Provider (in all auth backends)
        let res = match self.psw_backend {
            None => Err(AuthBackendError::NoRequestedBackend),
            Some(ref backend) => backend.get_user(user_id).await.map_err(AuthBackendError::from),
        };

        if res.is_ok() { return res }

        // TODO: simplify
        let res = match self.oauth2_backend {
            None => Err(AuthBackendError::NoRequestedBackend),
            Some(ref backend) => backend.get_user(user_id).await.map_err(AuthBackendError::from),
        };

        res
    }
}

pub type AuthSession = axum_login::AuthSession<AuthnBackend>;


#[derive(Debug, Clone, serde::Deserialize)]
pub enum AuthCredentials {
    Password(psw_auth::AuthCredentials),
    OAuth(oauth2_auth::AuthCredentials),
}
