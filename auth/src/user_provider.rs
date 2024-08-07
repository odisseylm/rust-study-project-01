use core::fmt;


#[axum::async_trait]
pub trait AuthUserProvider : fmt::Debug {
    type User: axum_login::AuthUser;
    async fn get_user_by_principal_identity(&self, user_id: &<Self::User as axum_login::AuthUser>::Id) -> Result<Option<Self::User>, AuthUserProviderError>;
}


#[derive(
    Debug,
    thiserror::Error,
    mvv_error_macro::ThisErrorFromWithBacktrace,
    mvv_error_macro::ThisErrorBacktraceSource,
)]
#[non_exhaustive]
pub enum AuthUserProviderError {
    // 1) It is used only for updates.
    // 2) If user is not found on get operation, just Ok(None) is returned.
    #[error("UserNotFound")]
    UserNotFound(UserId, BacktraceCell),

    #[error("Sqlx error")]
    Sqlx(#[source] #[from_with_bt] sqlx::Error, BacktraceCell),

    #[error("LockedResourceError")]
    LockedResourceError(BacktraceCell),

    #[error("ConfigurationError")]
    ConfigurationError(anyhow::Error),

    #[error("CacheError")]
    CacheError(anyhow::Error),

    #[error("UnknownError")]
    UnknownError(anyhow::Error),

    #[doc(hidden)]
    #[error("__NonExhaustive")]
    __NonExhaustive
}

/*
impl From<sqlx::Error> for AuthUserProviderError {
    fn from(value: sqlx::Error) -> Self {
        AuthUserProviderError::Sqlx(value)
    }
}
*/

impl From<CacheError> for AuthUserProviderError {
    fn from(err: CacheError) -> Self {
        match err {
            CacheError::CacheError(err) =>
                AuthUserProviderError::CacheError(err),
            err =>
                AuthUserProviderError::CacheError(anyhow::anyhow!(err))
        }
    }
}

impl From<CacheOrFetchError<AuthUserProviderError>> for AuthUserProviderError {
    fn from(value: CacheOrFetchError<AuthUserProviderError>) -> Self {
        match value {
            CacheOrFetchError::CacheError(err) =>
                AuthUserProviderError::CacheError(anyhow::Error::new(err)),
            CacheOrFetchError::FetchError(err) =>
                err,
            err =>
                AuthUserProviderError::CacheError(anyhow::Error::new(err)),
        }
    }
}



pub mod mem_user_provider;

pub use mem_user_provider::InMemAuthUserProvider;
use mvv_common::backtrace::BacktraceCell;
use mvv_common::cache::{CacheError, CacheOrFetchError};
use crate::UserId;
