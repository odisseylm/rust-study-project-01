use core::fmt;


#[derive(Clone)]
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(sqlx::FromRow)]
#[readonly::make]
pub struct AuthUser {
    pub id: i64,
    pub username: String,
    pub password: Option<String>,
    pub access_token: Option<String>,
}

impl AuthUser {
    pub fn new(id: i64, username: &'static str, password: &'static str) -> AuthUser {
        AuthUser { id, username: username.to_string(), password: Some(password.to_string()), access_token: None }
    }
    pub fn has_password<PswComparator: PasswordComparator>(&self, cred_psw: &str) -> bool {
        match self.password {
            None => false,
            Some(ref usr_psw) => {
                // usr_psw == cred_psw
                 PswComparator::passwords_equal(usr_psw, cred_psw)
            },
        }
    }
}


impl fmt::Debug for AuthUser {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("User0")
            .field("username", &self.username)
            .field("psw", &"[...]")
            .field("access_token", &"[...]")
            .finish()
    }
}

impl axum_login::AuthUser for AuthUser {
    type Id = String;

    fn id(&self) -> Self::Id {
        self.username.clone()
    }
    fn session_auth_hash(&self) -> &[u8] {
        if let Some(access_token) = &self.access_token {
            return access_token.as_bytes();
        }

        if let Some(password) = &self.password {
            // ???
            // We use the password hash as the auth hash -> what this means
            // is when the user changes their password the auth session becomes invalid.
            //
            return password.as_bytes();
        }

        &[]
    }
}


pub trait PasswordComparator {
    fn passwords_equal(user_password: &str, credentials_password: &str) -> bool;
}


#[derive(Clone)]
pub struct PlainPasswordComparator;
impl PasswordComparator for PlainPasswordComparator {
    fn passwords_equal(user_password: &str, credentials_password: &str) -> bool {
        user_password == credentials_password
    }
}


pub trait AuthUserProvider {
    type User: axum_login::AuthUser;
    fn get_user_by_name(&self, username: &str) -> Option<Self::User>;
    fn get_user_by_id(&self, user_id: &<AuthUser as axum_login::AuthUser>::Id) -> Option<Self::User>;
}
