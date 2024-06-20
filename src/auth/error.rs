use super::user_provider::AuthUserProviderError;
use super::permission::PermissionProcessError;


// This enum contains ALL possible errors for ANY auth Backend.
// Initially every impl had each own error enum... but I tired to convert them :-)
#[derive(Debug, thiserror::Error)]
pub enum AuthBackendError {
    // axum-login treats these cases as Ok(None)
    // We have to use the same approach in our code to conform idea.
    //
    // #[error("NoUser")]
    // NoUser,
    //
    // #[error("NoCredentials")]
    // NoCredentials,
    //
    // #[error("IncorrectUsernameOrPsw")]
    // IncorrectUsernameOrPsw,

    // ----------------------------------------------------------------------------
    //                            Internal errors
    //
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

    #[error("NoUserProvider")]
    NoUserProvider,

    #[error("DifferentUserProviders")]
    DifferentUserProviders,

    #[error(transparent)]
    TaskJoin(#[from] tokio::task::JoinError),

    #[error("ConfigError({0})")]
    ConfigError(anyhow::Error),

    #[error("RoleError({0})")]
    RoleError(PermissionProcessError),
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
impl From<PermissionProcessError> for AuthBackendError {
    fn from(value: PermissionProcessError) -> Self {
        AuthBackendError::RoleError(value)
    }
}
