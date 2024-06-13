
use core::fmt;
use std::collections::hash_map::HashMap;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
// use std::sync::{Arc, RwLock};
use std::sync::Arc;
use tokio::sync::RwLock;
// use std::hash::Hash;
use axum::body::Body;
use axum_extra::TypedHeader;
use axum_extra::headers::Authorization as AuthorizationHeader;
use axum_extra::headers::authorization::Basic;
// use axum_login::tower_sessions::service::{ CookieController, PlaintextCookie };
// use axum_login::tower_sessions::{ Expiry, MemoryStore, SessionManagerLayer };
use axum_login::tower_sessions::cookie::SameSite;
use time::Duration;
use crate::auth::auth_user::{AuthUserProvider, AuthUserProviderError};
use axum_login;
use axum_login::tower_sessions;
use crate::auth::oauth2_auth::Oauth2UserProvider;
use super::auth_user::AuthUser;
use super::psw::{ PasswordComparator, PlainPasswordComparator };


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
    // UP: AuthUserProvider<User = AuthUser> + Sync + Send, // + Clone + Sync + Send,
    PswComparator: PasswordComparator + Clone + Sync + Send,
    >(
    auth_session: AuthSession<PswComparator>,
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
    // UP: AuthUserProvider<User = AuthUser> + Clone + Sync + Send,
    PC: PasswordComparator + Clone + Sync + Send,
    >(
    auth_session: AuthSession<PC>, basic_auth_creds: Option<TypedHeader<AuthorizationHeader<Basic>>>,
    req: axum::extract::Request, next: axum::middleware::Next) -> http::Response<Body> {
    validate_auth_by_password(auth_session, basic_auth_creds, req, next).await
}


pub async fn validate_auth_by_password<
    // UP: AuthUserProvider<User = AuthUser> + Sync + Send, // + Clone + Sync + Send,
    PC: PasswordComparator + Clone + Sync + Send,
    >(
    auth_session: AuthSession<PC>,
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
    // UsrProvider,   // : AuthUserProvider<User = AuthUser> + Sync + Send, // + Clone + Sync + Send,
    PswComparator, // : PasswordComparator + Clone + Sync + Send,
> = axum_login::AuthSession<AuthBackend<PswComparator>>;


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
        self.route_layer(axum::middleware::from_fn(validate_auth_by_password::<PswComparator>))
    }
}

pub fn test_auth_by_psw_manager_layer()
    -> axum_login::AuthManagerLayer<AuthBackend<PlainPasswordComparator>, tower_sessions::MemoryStore> {

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
    // UsrProvider: AuthUserProvider<User = AuthUser> + Sync + Send,
    PswComparator: PasswordComparator + Clone + Sync + Send,
    SessionStore: tower_sessions::SessionStore,
    CookieController: tower_sessions::service::CookieController,
    >(
    // user_provider: Arc<UsrProvider>,
    user_provider: Arc<dyn AuthUserProvider<User = AuthUser> + Sync + Send>,
    session_manager_layer: tower_sessions::SessionManagerLayer<SessionStore, CookieController>,
    // ) -> axum_login::AuthManagerLayer<AuthBackend<UsrProvider, PswComparator>, tower_sessions::MemoryStore> {
    ) -> axum_login::AuthManagerLayer<AuthBackend<PswComparator>, SessionStore, CookieController> {

    // Auth service.
    //
    // This combines the session layer with our backend to establish the auth
    // service which will provide the auth session as a request extension.
    let backend: AuthBackend<PswComparator> = AuthBackend::new(user_provider);
    axum_login::AuthManagerLayerBuilder::new(backend, session_manager_layer).build()
}


// #[derive(Clone)]
pub struct AuthBackend<
    // UserProvider: AuthUserProvider<User = AuthUser> + Sync + Send, // + Clone + Sync + Send,
    PswComparator: PasswordComparator + Clone + Sync + Send,
> {
    users_provider: Arc<dyn AuthUserProvider<User = AuthUser> + Sync + Send>,
    _pd: PhantomData<PswComparator>,
}
impl<
    // UserProvider: AuthUserProvider<User = AuthUser> + Sync + Send, // + Clone + Sync + Send,
    PswComparator: PasswordComparator + Clone + Sync + Send,
> Clone for AuthBackend<PswComparator> {
    fn clone(&self) -> Self {
        AuthBackend::<PswComparator> {
            users_provider: self.users_provider.clone(),
            _pd: PhantomData,
        }
    }
    fn clone_from(&mut self, source: &Self) {
        self.users_provider = source.users_provider.clone();
    }
}


struct InMemoryState {
    // TODO: I think we could use there Rc (instead of Arc) because it is protected by mutex... but how to say rust about it??
    // TODO: RwLock TWICE?? It is too much!!!
    users_by_username: HashMap<String, Arc<RwLock<AuthUser>>>,
    users_by_id: HashMap<i64, Arc<RwLock<AuthUser>>>,
}
impl InMemoryState {
    fn new() -> InMemoryState {
        InMemoryState {
            users_by_username: HashMap::<String, Arc<RwLock<AuthUser>>>::new(),
            users_by_id: HashMap::<i64, Arc<RwLock<AuthUser>>>::new(),
        }
    }
    fn with_capacity(capacity: usize) -> InMemoryState {
        InMemoryState {
            users_by_username: HashMap::<String, Arc<RwLock<AuthUser>>>::with_capacity(capacity),
            users_by_id: HashMap::<i64, Arc<RwLock<AuthUser>>>::with_capacity(capacity),
        }
    }
}


// #[derive(Clone, Debug)]
#[derive(Clone)]
pub struct TestAuthUserProvider {
    // state: Arc<Mutex<InMemoryState>>,
    state: Arc<RwLock<InMemoryState>>,
}
impl TestAuthUserProvider {
    pub fn new() -> TestAuthUserProvider {
        TestAuthUserProvider {
            // state: Arc::new(Mutex::<InMemoryState>::new(InMemoryState::new())),
            state: Arc::new(RwLock::<InMemoryState>::new(InMemoryState::new())),
        }
    }

    pub async fn with_users(users: Vec<AuthUser>) -> Result<TestAuthUserProvider, AuthUserProviderError> {
        let in_memory_state = {
            // let in_memory_state = Arc::new(Mutex::<InMemoryState>::new(InMemoryState::with_capacity(users.len())));
            let in_memory_state = Arc::new(RwLock::<InMemoryState>::new(InMemoryState::with_capacity(users.len())));
            // let mut guarded = in_memory_state.lock()
            {
                let mut guarded = in_memory_state.deref().write() // get_mut()
                    // .map_err(|_|AuthUserProviderError::LockedResourceError) ?;
                    .await;

                for user in users {
                    let user_ref = Arc::new(RwLock::new(user.clone()));

                    guarded.users_by_id.insert(user.id, Arc::clone(&user_ref));
                    guarded.users_by_username.insert(user.username.to_string(), Arc::clone(&user_ref));
                }
            //forget(guarded); // !!! 'forget' is risky function !!??!! It does NOT work!!
            }

            in_memory_state
        };

        Ok(TestAuthUserProvider {
            state: in_memory_state,
        })
    }

    async fn test_users() -> Result<TestAuthUserProvider, AuthUserProviderError> {
        Self::with_users(vec!(AuthUser::new(1, "vovan", "qwerty"))).await
    }
}


impl fmt::Debug for TestAuthUserProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO: how to write it for async ??
        write!(f, "TestAuthUserProvider {{ ... }}")

        /*
        // let state_res = self.state.lock();
        let state_res = self.state.deref().read();
        match state_res {
            Ok(ref state) => {
                let users = state.users_by_username.keys().map(|el|el.clone()).collect::<Vec<String>>().join(", ");
                write!(f, "TestAuthUserProvider {{ {} }}", users)
            }
            Err(_) => write!(f, "TestAuthUserProvider {{ Inaccessible content }}"),
        }
        */
    }
}

#[axum::async_trait]
impl AuthUserProvider for TestAuthUserProvider {
    type User = AuthUser;
    async fn get_user_by_name(&self, username: &str) -> Result<Option<Self::User>, AuthUserProviderError> {
        let state_res = rw_lock_ok(self.state.read().await);
        match state_res {
            Err(_)    => Err(AuthUserProviderError::LockedResourceError),
            Ok(state) => {
                let map_value = state.users_by_username.get(username);
                match map_value {
                    None => Ok(None),
                    Some(map_value) => {
                        match rw_lock_ok(map_value.read().await) {
                            Err(_) => Err(AuthUserProviderError::LockedResourceError),
                            Ok(v)  => Ok(Some(v.deref().clone())),
                        }
                    }
                }
            }
        }
    }

    async fn get_user_by_id(&self, user_id: &<AuthUser as axum_login::AuthUser>::Id) -> Result<Option<Self::User>, AuthUserProviderError> {
        let state_res = rw_lock_ok(self.state.read().await);
        match state_res {
            Err(_)    => Err(AuthUserProviderError::LockedResourceError),
            Ok(state) => {
                // Ok(state.users_by_username.get(username).map(|usr| usr.deref().clone()))
                let map_value = state.users_by_id.get(user_id);
                match map_value {
                    None => Ok(None),
                    Some(map_value) => {
                        match rw_lock_ok(map_value.read().await) {
                            Err(_) => Err(AuthUserProviderError::LockedResourceError),
                            Ok(v)  => Ok(Some(v.deref().clone())),
                        }
                    }
                }
            }
        }
    }
}

fn rw_lock_ok<T>(t: T) -> Result<T, AuthUserProviderError> {
    Ok(t)
}

#[axum::async_trait]
impl Oauth2UserProvider for TestAuthUserProvider {
    // type User = AuthUser;

    async fn update_user_access_token(&self, username: &str, secret_token: &str) -> Result<Option<Self::User>, AuthUserProviderError> {
        let state_res = rw_lock_ok(self.state.write().await);
        match state_res {
            Err(_)    => Err(AuthUserProviderError::LockedResourceError),
            Ok(state) => {
                // Ok(state.users_by_username.get(username).map(|usr| usr.deref().clone()))
                let map_value = state.users_by_username.get(username);
                match map_value {
                    None => Ok(None),
                    Some(map_value) => {
                        match rw_lock_ok(map_value.write().await) {
                            Err(_) => Err(AuthUserProviderError::LockedResourceError),
                            Ok(ref mut v)  => {
                                v.deref_mut().access_token(Some(secret_token.to_string()));
                                Ok(Some(v.clone()))
                            },
                        }
                    }
                }
            }
        }
    }
}

impl<PswComparator> AuthBackend<PswComparator> where
    // UserProvider: AuthUserProvider<User = AuthUser> + Sync + Send, // + Clone + Sync + Send,
    PswComparator: PasswordComparator + Clone + Sync + Send,
{
    pub fn new(users_provider: Arc<dyn AuthUserProvider<User = AuthUser> + Sync + Send>) -> AuthBackend<PswComparator> {
        AuthBackend::<PswComparator> { users_provider: users_provider.clone(), _pd: PhantomData }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("NoUser")]
    NoUser,
    #[error("IncorrectUsernameOrPsw")]
    IncorrectUsernameOrPsw,
    #[error("UserProviderError")]
    UserProviderError(AuthUserProviderError),
}

impl From<AuthUserProviderError> for AuthError {
    fn from(value: AuthUserProviderError) -> Self {
        AuthError::UserProviderError(value)
    }
}


#[axum::async_trait]
impl<
    // UserProvider: AuthUserProvider<User = AuthUser> + Sync + Send, // + Clone + Sync + Send,
    PswComparator: PasswordComparator + Clone + Sync + Send,
    > axum_login::AuthnBackend for AuthBackend<PswComparator> {
    type User = AuthUser;
    type Credentials = AuthCredentials;
    type Error = AuthError;

    async fn authenticate(&self, creds: Self::Credentials) -> Result<Option<Self::User>, Self::Error> {
        let usr_res = self.users_provider.get_user_by_name(creds.username.as_str()).await;

        let usr_opt = match usr_res {
            Ok(usr_opt) => usr_opt,
            Err(err) => return Err(Self::Error::UserProviderError(err))
        };

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
        let usr_opt_res = self.users_provider.get_user_by_id(user_id).await.map_err(From::<AuthUserProviderError>::from);
        usr_opt_res
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



#[cfg(test)]
mod tests {
    use crate::util::{TestOptionUnwrap, TestResultUnwrap};
    use super::*;

    macro_rules! aw {
      ($e:expr) => {
          tokio_test::block_on($e)
      };
    }

    async fn some_async_fn_1() -> i32 {
        123
    }
    async fn some_async_fn_2() -> i32 {
        some_async_fn_1().await * 2
    }

    #[tokio::test]
    #[allow(non_snake_case)]
    async fn tests_TestAuthUserProvider() {

        let aa = async { 123 }.await;
        println!("aa: {}", aa);

        let bb = some_async_fn_2().await;
        println!("bb: {}", bb);

        let users = TestAuthUserProvider::test_users().await.test_unwrap();

        // -----------------------------------------------------------------------------------------
        let usr_opt_res = users.get_user_by_id(&1i64).await;

        assert!(usr_opt_res.is_ok()); // no error
        let usr_opt = usr_opt_res.test_unwrap();
        assert!(usr_opt.is_some()); // and user exists

        let usr = usr_opt.test_unwrap();
        assert_eq!(usr.id, 1i64);
        assert_eq!(usr.username.as_str(), "vovan");
        assert_eq!(usr.password, Some("qwerty".to_string()));
        assert_eq!(usr.access_token, None);

        // -----------------------------------------------------------------------------------------
        let usr_opt_res = users.update_user_access_token("vovan", "token1").await;
        println!("### usr_opt_res: {:?}", usr_opt_res);

        assert!(usr_opt_res.is_ok()); // no error
        let usr_opt = usr_opt_res.test_unwrap();
        assert!(usr_opt.is_some()); // and user exists

        let usr = usr_opt.test_unwrap();
        assert_eq!(usr.id, 1i64);
        assert_eq!(usr.username.as_str(), "vovan");
        assert_eq!(usr.password, Some("qwerty".to_string()));
        assert_ne!(usr.access_token, None);
        assert_eq!(usr.access_token, Some("token1".to_string()));

        // -----------------------------------------------------------------------------------------
        let usr_opt_res = users.get_user_by_id(&1i64).await;

        assert!(usr_opt_res.is_ok()); // no error
        let usr_opt = usr_opt_res.test_unwrap();
        assert!(usr_opt.is_some()); // and user exists

        let usr = usr_opt.test_unwrap();
        assert_eq!(usr.id, 1i64);
        assert_eq!(usr.username.as_str(), "vovan");
        assert_eq!(usr.password, Some("qwerty".to_string()));
        assert_ne!(usr.access_token, None);
        assert_eq!(usr.access_token, Some("token1".to_string()));

        println!("Test tests_TestAuthUserProvider is completed.")
    }

    #[tokio::test]
    async fn test_6565() {
        let lock = Arc::new(RwLock::new(5));

        // many reader locks can be held at once
        {
            let r1 = lock.read().await;
            let r2 = lock.read().await;
            assert_eq!(*r1, 5);
            assert_eq!(*r2, 5);
        } // read locks are dropped at this point

        // only one write lock may be held, however
        {
            let mut w = lock.write().await;
            *w += 1;
            assert_eq!(*w, 6);
        } // write lock is dropped here
    }
}
