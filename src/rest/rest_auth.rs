use core::fmt;
use std::collections::hash_map::HashMap;
use std::fmt::Formatter;
use std::hash::Hash;
use axum_login::AuthManagerLayer;
use axum_login::tower_sessions::{MemoryStore, SessionManagerLayer};
// use axum_login::{AuthManagerLayerBuilder, UserId};


pub fn auth_manager_layer() -> AuthManagerLayer<AuthnBackend0, MemoryStore> {

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
    let backend = AuthnBackend0::new();
    let auth_layer: AuthManagerLayer<AuthnBackend0, MemoryStore> = AuthManagerLayerBuilder::new(backend, session_layer).build();
    auth_layer
}


// pub fn auth_manager_layer22<
//         Sessions: axum_login::tower_sessions::SessionStore,
//         C: axum_login::tower_sessions::service::CookieController,
//     >(
//     session_manager_layer: SessionManagerLayer<Sessions, C>,
//     ) -> AuthManagerLayer<AuthnBackend0, Sessions, C> {
//
//     let aa = axum_login::AuthManagerLayerBuilder::new(
//         AuthnBackend0::new(),
//         session_manager_layer,
//     ).build();
//     aa
// }


#[derive(Clone)]
pub struct AuthnBackend0 {
    users: HashMap<String, User0>,
}

impl AuthnBackend0 {
    fn new() -> AuthnBackend0 {
        AuthnBackend0 {
            users: {
                let mut users = HashMap::<String, User0>::with_capacity(2);
                users.insert("vovan".to_string(), User0::new("vovan", "qwerty"));
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
impl axum_login::AuthnBackend for AuthnBackend0 {
    type User = User0;
    type Credentials = Cred0;
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


#[derive(Clone)]
pub struct User0 {
    username: String,
    psw: String,
}

impl User0 {
    fn new(username: &'static str, psw: &'static str) -> User0 {
        User0 { username: username.to_string(), psw: psw.to_string() }
    }
}

impl fmt::Debug for User0 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("User0")
            .field("username", &self.username)
            .field("psw", &"[...]")
            .finish()
    }
}

impl axum_login::AuthUser for User0 {
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
pub struct Cred0 {
    pub username: String,
    pub password: String,
}

impl fmt::Debug for Cred0 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Cred0 {{ username: {:?}, psw: [...] }},", self.username)
    }
}
