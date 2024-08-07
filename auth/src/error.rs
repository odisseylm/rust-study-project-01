use axum::response::{ IntoResponse, Response };
use http::StatusCode;
use log::error;
use mvv_common::backtrace::BacktraceCell;
use crate::backend::Oauth2ConfigError;

use crate::user_provider::AuthUserProviderError;
use crate::permission::PermissionProcessError;
//--------------------------------------------------------------------------------------------------


// This enum contains ALL possible errors for ANY auth Backend.
// Initially every impl had each own error enum... but I tired to convert them :-)
#[derive(
    Debug,
    thiserror::Error,
    mvv_error_macro::ThisErrorFromWithBacktrace,
    mvv_error_macro::ThisErrorBacktraceSource,
)]
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
    #[error("User Provider error")]
    UserProviderError(#[from] AuthUserProviderError),

    #[error("Sqlx error")]
    Sqlx(#[source] #[from_with_bt] sqlx::Error, BacktraceCell),

    #[error("Reqwest error")]
    Reqwest(#[source] #[from_with_bt] reqwest::Error, BacktraceCell),

    #[error("OAuth2 error")]
    OAuth2(#[source] #[from_with_bt] oauth2::basic::BasicRequestTokenError<oauth2::reqwest::AsyncHttpClientError>, BacktraceCell),

    #[error("OAuth2 config error")]
    OAuth2ConfigError(#[from] Oauth2ConfigError),

    #[error("No RequestedBackend error")]
    NoRequestedBackend(BacktraceCell),

    #[error("No UserProvider error")]
    NoUserProvider(BacktraceCell),

    #[error("NoPermission provider error")]
    NoPermissionProvider(BacktraceCell),

    #[error("Different UserProviders configuration error")]
    DifferentUserProviders(BacktraceCell),

    #[error("Different PermissionProviders configuration error")]
    DifferentPermissionProviders(BacktraceCell),

    #[error("TaskJoin error")]
    TaskJoin(#[source] #[from_with_bt] tokio::task::JoinError, BacktraceCell),

    #[error("Config error: {0}")]
    ConfigError(#[source] anyhow::Error),

    #[error("RoleError: {0}")]
    RoleError(#[source] #[from_with_bt] PermissionProcessError, BacktraceCell),

    #[doc(hidden)]
    #[error("__NonExhaustive")]
    __NonExhaustive
}

/*
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
*/


impl IntoResponse for AuthBackendError {
    fn into_response(self) -> Response {
        // T O D O: Probably logging is should be done in other place.
        error!("Internal error: {}", self);
        StatusCode::INTERNAL_SERVER_ERROR.into_response()

        /*
        match self {
            AuthBackendError::UserProviderError(_) => {}
            AuthBackendError::Sqlx(_) => {}
            AuthBackendError::Reqwest(_) => {}
            AuthBackendError::OAuth2(_) => {}
            AuthBackendError::NoRequestedBackend => {}
            AuthBackendError::NoUserProvider => {}
            AuthBackendError::DifferentUserProviders => {}
            AuthBackendError::TaskJoin(_) => {}
            AuthBackendError::ConfigError(_) => {}
            AuthBackendError::RoleError(_) => {}
        }
        */
    }
}
