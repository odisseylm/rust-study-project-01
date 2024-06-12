
use core::fmt;
use std::collections::hash_map::HashMap;
use std::marker::PhantomData;
use std::sync::Arc;
// use std::hash::Hash;
use axum::body::Body;
use axum_extra::TypedHeader;
use axum_extra::headers::Authorization as AuthorizationHeader;
use axum_extra::headers::authorization::Basic;
// use axum_login::tower_sessions::service::{ CookieController, PlaintextCookie };
// use axum_login::tower_sessions::{ Expiry, MemoryStore, SessionManagerLayer };
use axum_login::tower_sessions::cookie::SameSite;
use time::Duration;
use crate::auth::auth_user_provider::AuthUserProvider;
use axum_login;
use axum_login::tower_sessions;
use super::auth_user_provider::AuthUser;
use super::auth_user_provider::{ PasswordComparator, PlainPasswordComparator };


mod tw {
    // type axum_login::tower_sessions::SessionStore = SessionStore;
    // type SessionStore = axum_login::tower_sessions::SessionStore;
}

// pub mod auth {
//     // pub use super::AuthUser;
//     pub use super::validate_auth;
//
//     pub type AuthUser = super::AuthUser;
//     pub type AuthnBackend = super::AuthnBackend;
//     pub type AuthSession = super::AuthSession;
// }

// trait PasswordComparator {
//     fn passwords_equal(user_password: &str, credentials_password: &str) -> bool;
// }
//
//
// #[derive(Clone)]
// struct PlainPasswordComparator;
// impl PasswordComparator for PlainPasswordComparator {
//     fn passwords_equal(user_password: &str, credentials_password: &str) -> bool {
//         user_password == credentials_password
//     }
// }


// TODO: how to avoid this AuthUserProvider? It is really unneeded there.
//       Or use some '_' or '*' or '?'...
async fn is_authenticated<
    UP: AuthUserProvider<User = AuthUser> + Sync + Send, // + Clone + Sync + Send,
    PswComparator: PasswordComparator + Clone + Sync + Send,
    >(
    auth_session: AuthSession<UP,PswComparator>,
    basic_auth_creds: Option<TypedHeader<AuthorizationHeader<Basic>>>,
    ) -> bool {

    if auth_session.user.is_some() {
        return true;
    }

    if let Some(TypedHeader(AuthorizationHeader(ref creds))) = basic_auth_creds {
        // T O D O: avoid to_string()
        let is_auth_res = auth_session.authenticate(AuthCredentials {
            username: creds.username().to_string(),
            password: creds.password().to_string(),
            next: None,
        }).await;
        is_auth_res.is_ok()
    }
    else { false }
}


#[inline] // TODO: remove from there
pub async fn validate_auth_temp<
    UP: AuthUserProvider<User = AuthUser> + Clone + Sync + Send,
    PC: PasswordComparator + Clone + Sync + Send,
    >(
    auth_session: AuthSession<UP,PC>, basic_auth_creds: Option<TypedHeader<AuthorizationHeader<Basic>>>,
    req: axum::extract::Request, next: axum::middleware::Next) -> http::Response<Body> {
    validate_auth_by_password(auth_session, basic_auth_creds, req, next).await
}


pub async fn validate_auth_by_password<
    UP: AuthUserProvider<User = AuthUser> + Sync + Send, // + Clone + Sync + Send,
    PC: PasswordComparator + Clone + Sync + Send,
    >(
    auth_session: AuthSession<UP,PC>,
    basic_auth_creds: Option<TypedHeader<AuthorizationHeader<Basic>>>,
    req: axum::extract::Request,
    next: axum::middleware::Next
    ) -> http::Response<Body> {

    if is_authenticated(auth_session, basic_auth_creds).await {
        next.run(req).await
    } else {
        // or redirect to login page
        // should be configurable outside: dev or prod
        crate::rest::error_rest::unauthenticated_401_response()
    }
}

pub type AuthSession<
    UsrProvider: AuthUserProvider<User = AuthUser> + Sync + Send, // + Clone + Sync + Send,
    PswComparator: PasswordComparator + Clone + Sync + Send,
> = axum_login::AuthSession<AuthBackend<UsrProvider,PswComparator>>;


// TODO: remove from there
#[extension_trait::extension_trait]
pub impl<
    S: Clone + Send + Sync + 'static,
    > AuthenticationByPasswordRequiredExtension for axum::Router<S> {
    // #[inline] // warning: `#[inline]` is ignored on function prototypes
    #[track_caller]
    fn auth_by_password_required<
        UsrProvider: AuthUserProvider<User = AuthUser> + Sync + Send + 'static, // + Clone + Sync + Send + 'static,
        PswComparator: PasswordComparator + Clone + Sync + Send + 'static,
        >(self) -> Self {
        self.route_layer(axum::middleware::from_fn(validate_auth_by_password::<UsrProvider, PswComparator>))
    }
}

pub fn test_auth_by_psw_manager_layer()
    -> axum_login::AuthManagerLayer<AuthBackend<TestAuthUserProvider, PlainPasswordComparator>, tower_sessions::MemoryStore> {

    use tower_sessions::{ MemoryStore, service::SessionManagerLayer, Expiry };

    let user_provider = Arc::new(TestAuthUserProvider::new());
    let session_layer = SessionManagerLayer::new(MemoryStore::default())
        .with_secure(false)
        .with_same_site(SameSite::Lax) // Ensure we send the cookie from the OAuth redirect.
        .with_expiry(Expiry::OnInactivity(Duration::days(1)));

    build_auth_by_psw_manager_layer(user_provider, session_layer)
}


pub fn build_auth_by_psw_manager_layer<
    // UsrProvider: AuthUserProvider<User = AuthUser> + Clone + Sync + Send,
    UsrProvider: AuthUserProvider<User = AuthUser> + Sync + Send,
    PswComparator: PasswordComparator + Clone + Sync + Send,
    SessionStore: tower_sessions::SessionStore,
    CookieController: tower_sessions::service::CookieController,
    >(
    user_provider: Arc<UsrProvider>,
    session_manager_layer: tower_sessions::SessionManagerLayer<SessionStore, CookieController>,
    // ) -> axum_login::AuthManagerLayer<AuthBackend<UsrProvider, PswComparator>, tower_sessions::MemoryStore> {
    ) -> axum_login::AuthManagerLayer<AuthBackend<UsrProvider, PswComparator>, SessionStore, CookieController> {

    // Auth service.
    //
    // This combines the session layer with our backend to establish the auth
    // service which will provide the auth session as a request extension.
    let backend: AuthBackend<UsrProvider, PswComparator> = AuthBackend::new(user_provider);
    axum_login::AuthManagerLayerBuilder::new(backend, session_manager_layer).build()
}


// #[derive(Clone)]
pub struct AuthBackend<
    UserProvider: AuthUserProvider<User = AuthUser> + Sync + Send, // + Clone + Sync + Send,
    PswComparator: PasswordComparator + Clone + Sync + Send,
> {
    users_provider: Arc<UserProvider>,
    _pd: PhantomData<PswComparator>,
}
impl<
    UserProvider: AuthUserProvider<User = AuthUser> + Sync + Send, // + Clone + Sync + Send,
    PswComparator: PasswordComparator + Clone + Sync + Send,
> Clone for AuthBackend<UserProvider, PswComparator> {
    fn clone(&self) -> Self {
        AuthBackend::<UserProvider, PswComparator> {
            users_provider: self.users_provider.clone(),
            _pd: PhantomData,
        }
    }
    fn clone_from(&mut self, source: &Self) {
        self.users_provider = source.users_provider.clone();
    }
}


#[derive(Clone)]
struct TestAuthUserProvider { // TODO: use Arc
    users_by_username: HashMap<String, AuthUser>,
    users_by_id: HashMap<i64, AuthUser>,
}
impl TestAuthUserProvider {
    fn new() -> TestAuthUserProvider {
        let user_1 = AuthUser::new(1, "vovan", "qwerty");
        TestAuthUserProvider {
            users_by_username: {
                let mut users = HashMap::<String, AuthUser>::with_capacity(2);
                users.insert(user_1.username.clone(), user_1.clone());
                users
            },
            users_by_id: {
                let mut users = HashMap::<i64, AuthUser>::with_capacity(2);
                users.insert(user_1.id, user_1.clone());
                users
            },
        }
    }
}
impl AuthUserProvider for TestAuthUserProvider {
    type User = AuthUser;
    fn get_user_by_name(&self, username: &str) -> Option<Self::User> {
        self.users_by_username.get(username).map(|usr|usr.clone())
    }
    // fn get_user_by_id(&self, user_id: &Self::User::Id) -> Option<Self::User> {
    fn get_user_by_id(&self, user_id: &<AuthUser as axum_login::AuthUser>::Id) -> Option<Self::User> {
        self.users_by_username.get(user_id).map(|usr|usr.clone())
    }
}


impl<UserProvider, PswComparator> AuthBackend<UserProvider, PswComparator> where
    UserProvider: AuthUserProvider<User = AuthUser> + Sync + Send, // + Clone + Sync + Send,
    PswComparator: PasswordComparator + Clone + Sync + Send,
{
    fn new(users_provider: Arc<UserProvider>) -> AuthBackend<UserProvider, PswComparator> {
        AuthBackend::<UserProvider, PswComparator> { users_provider: users_provider.clone(), _pd: PhantomData }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("NoUser")]
    NoUser,
    #[error("IncorrectUsernameOrPsw")]
    IncorrectUsernameOrPsw,
    #[error("UserProviderError")]
    UserProviderError,
}


#[axum::async_trait]
impl<
    UserProvider: AuthUserProvider<User = AuthUser> + Sync + Send, // + Clone + Sync + Send,
    PswComparator: PasswordComparator + Clone + Sync + Send,
    > axum_login::AuthnBackend for AuthBackend<UserProvider, PswComparator> {
    type User = AuthUser;
    type Credentials = AuthCredentials;
    type Error = AuthError;

    async fn authenticate(&self, creds: Self::Credentials) -> Result<Option<Self::User>, Self::Error> {
        let usr_opt = self.users_provider.get_user_by_name(creds.username.as_str());
        match usr_opt {
            None => Err(Self::Error::NoUser),
            Some(usr) => {
                let usr_psw = usr.password.as_ref().map(|s|s.as_str()).unwrap_or("");
                // if usr.username == creds.username && usr.psw == creds.password {
                if usr.username == creds.username &&
                    PswComparator::passwords_equal(usr_psw, creds.password.as_str()) {
                    Ok(Some(usr.clone()))
                } else {
                    Err(Self::Error::IncorrectUsernameOrPsw)
                }
            }
        }
    }

    async fn get_user(&self, user_id: &axum_login::UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        // TODO: what is UserId there???
        let usr_opt = self.users_provider.get_user_by_name(user_id.as_str());
        match usr_opt {
            None => Ok(None),
            Some(user) => Ok(Some(user.clone()))
        }
    }
}

#[derive(Clone, serde::Deserialize)]
pub struct AuthCredentials {
    pub username: String,
    pub password: String,
    // seems it source/initial page... It is a bit bad design, but...
    pub next: Option<String>,
}

impl fmt::Debug for AuthCredentials {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Cred0 {{ username: {:?}, psw: [...] }},", self.username)
    }
}
