use core::fmt;
use std::collections::hash_map::HashMap;
// use std::hash::Hash;
use axum::body::Body;
use axum_extra::TypedHeader;
use axum_extra::headers::Authorization as AuthorizationHeader;
use axum_extra::headers::authorization::Basic;

// pub mod auth {
//     // pub use super::AuthUser;
//     pub use super::validate_auth;
//
//     pub type AuthUser = super::AuthUser;
//     pub type AuthnBackend = super::AuthnBackend;
//     pub type AuthSession = super::AuthSession;
// }


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


#[derive(Clone)]
pub struct AuthnBackend {
    users: HashMap<String, AuthUser>,
}

impl AuthnBackend {
    fn new() -> AuthnBackend {
        AuthnBackend {
            users: {
                let mut users = HashMap::<String, AuthUser>::with_capacity(2);
                users.insert("vovan".to_string(), AuthUser::new("vovan", "qwerty"));
                users
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("NoUser")]
    NoUser,
    #[error("IncorrectUsernameOrPsw")]
    IncorrectUsernameOrPsw,
}


#[axum::async_trait]
impl axum_login::AuthnBackend for AuthnBackend {
    type User = AuthUser;
    type Credentials = AuthCredentials;
    type Error = AuthError;

    async fn authenticate(&self, creds: Self::Credentials) -> Result<Option<Self::User>, Self::Error> {
        let usr_opt = self.users.get(creds.username.as_str());
        match usr_opt {
            None => Err(Self::Error::NoUser),
            Some(usr) =>
                if usr.username == creds.username && usr.psw == creds.password {
                    Ok(Some(usr.clone()))
                } else {
                    Err(Self::Error::IncorrectUsernameOrPsw)
                },
        }
    }

    async fn get_user(&self, user_id: &axum_login::UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        let usr_opt = self.users.get(user_id.as_str());
        match usr_opt {
            None => Ok(None),
            Some(user) => Ok(Some(user.clone()))
        }
    }
}

pub type AuthSession = axum_login::AuthSession<AuthnBackend>;

#[derive(Clone)]
pub struct AuthUser {
    username: String,
    psw: String,
}

impl AuthUser {
    fn new(username: &'static str, psw: &'static str) -> AuthUser {
        AuthUser { username: username.to_string(), psw: psw.to_string() }
    }
}

impl fmt::Debug for AuthUser {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("User0")
            .field("username", &self.username)
            .field("psw", &"[...]")
            .finish()
    }
}

impl axum_login::AuthUser for AuthUser {
    type Id = String;

    fn id(&self) -> Self::Id {
        self.username.clone()
    }
    fn session_auth_hash(&self) -> &[u8] {
        // if let Some(access_token) = &self.access_token {
        //     return access_token.as_bytes();
        // }
        //
        // if let Some(password) = &self.psw {
        //     return password.as_bytes();
        // }
        //
        // &[]
        self.psw.as_bytes() // TODO: hm..??
    }
}

#[derive(Clone, serde::Deserialize)]
pub struct AuthCredentials {
    pub username: String,
    pub password: String,
}

impl fmt::Debug for AuthCredentials {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Cred0 {{ username: {:?}, psw: [...] }},", self.username)
    }
}
