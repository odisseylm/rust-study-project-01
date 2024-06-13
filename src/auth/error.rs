use super::auth_user_provider::AuthUserProviderError;


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

impl From<sqlx::Error> for AuthBackendError {
    fn from(value: sqlx::Error) -> Self {
        AuthBackendError::Sqlx(value)
    }
}
