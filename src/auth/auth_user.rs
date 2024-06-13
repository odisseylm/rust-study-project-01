use core::fmt;
use super::psw::PasswordComparator;


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
    pub fn access_token(&mut self, access_token: Option<String>) {
        self.access_token = access_token;
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
    type Id = i64;

    fn id(&self) -> Self::Id {
        self.id
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


#[axum::async_trait]
pub trait AuthUserProvider : fmt::Debug {
    type User: axum_login::AuthUser;
    async fn get_user_by_name(&self, username: &str) -> Result<Option<Self::User>, AuthUserProviderError>;
    async fn get_user_by_id(&self, user_id: &<AuthUser as axum_login::AuthUser>::Id) -> Result<Option<Self::User>, AuthUserProviderError>;
}


#[derive(Debug, thiserror::Error)]
pub enum AuthUserProviderError {
    // 1) It is used only for updates.
    // 2) If user is not found on get operation, just Ok(None) is returned.
    #[error("UserNotFound")]
    UserNotFound,

    #[error(transparent)]
    Sqlx(sqlx::Error),

    #[error("LockedResourceError")]
    LockedResourceError,
}


// This enum contains ALL possible errors for ANY auth Backend.
// Initially every impl had each own error enum... but I tired to convert them :-)
#[derive(Debug, thiserror::Error)]
pub enum AuthBackendError {
    #[error("NoUser")]
    NoUser,

    #[error("IncorrectUsernameOrPsw")]
    IncorrectUsernameOrPsw,

    #[error("UserProviderError")]
    UserProviderError(AuthUserProviderError),

    #[error(transparent)]
    Sqlx(sqlx::Error),

    #[error(transparent)]
    Reqwest(reqwest::Error),

    #[error(transparent)]
    OAuth2(oauth2::basic::BasicRequestTokenError<oauth2::reqwest::AsyncHttpClientError>),

    #[error("NoRequestedBackend")]
    NoRequestedBackend,

    #[error(transparent)]
    TaskJoin(#[from] tokio::task::JoinError),
}

impl From<AuthUserProviderError> for AuthBackendError {
    fn from(value: AuthUserProviderError) -> Self {
        AuthBackendError::UserProviderError(value)
    }
}
