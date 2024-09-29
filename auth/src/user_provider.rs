use core::fmt::Debug;
use mvv_common::{
    backtrace::BacktraceCell,
    cache::{CacheError, CacheOrFetchError},
};
use crate::UserId;
//--------------------------------------------------------------------------------------------------


#[axum::async_trait]
pub trait AuthUserProvider : Debug {
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
    // It is bad Idea to make visible outside extern dependencies
    // due to rust version crate interpretation.
    // Sqlx(#[source] #[from_with_bt] sqlx::Error, BacktraceCell),
    DatabaseError(#[source] Box<dyn std::error::Error + Send + Sync>, BacktraceCell),

    #[error("LockedResourceError")]
    LockedResourceError(BacktraceCell),

    #[error("ConfigurationError")]
    ConfigurationError(anyhow::Error),

    #[error("CacheError")]
    CacheError(anyhow::Error), // TODO: replace anyhow by Box<dyn StdError>

    #[error("UnknownError")]
    UnknownError(anyhow::Error),

    #[doc(hidden)]
    #[error("__NonExhaustive")]
    __NonExhaustive
}
impl AuthUserProviderError {
    #[inline]
    #[track_caller]
    pub fn user_not_found_err(user_id: UserId) -> Self {
        Self::UserNotFound(user_id, backtrace())
    }
    #[inline]
    #[track_caller]
    pub fn database_err<E: std::error::Error + Send + Sync + 'static>(err: E) -> Self {
        Self::DatabaseError(Box::new(err), backtrace())
    }
    // now can be used only for breakpoint
    #[inline]
    #[track_caller]
    pub fn sqlx_err<E: std::error::Error + Send + Sync + 'static>(err: E) -> Self {
        Self::database_err(err)
    }
    // now can be used only for breakpoint
    #[inline]
    #[track_caller]
    pub fn diesel_err<E: std::error::Error + Send + Sync + 'static>(err: E) -> Self {
        Self::database_err(err)
    }
    #[inline]
    #[track_caller]
    pub fn locked_resource_err() -> Self {
        Self::LockedResourceError(backtrace())
    }
    #[inline]
    #[track_caller]
    pub fn cfg_err(err: anyhow::Error) -> Self {
        Self::ConfigurationError(err)
    }
    #[inline]
    #[track_caller]
    pub fn cache_err(err: anyhow::Error) -> Self {
        Self::CacheError(err)
    }
    #[inline]
    #[track_caller]
    pub fn unknown_err(err: anyhow::Error) -> Self {
        Self::UnknownError(err)
    }
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
                AuthUserProviderError::cache_err(err),
            err =>
                AuthUserProviderError::cache_err(anyhow::anyhow!(err))
        }
    }
}

impl From<CacheOrFetchError<AuthUserProviderError>> for AuthUserProviderError {
    fn from(value: CacheOrFetchError<AuthUserProviderError>) -> Self {
        match value {
            CacheOrFetchError::CacheError(err) =>
                AuthUserProviderError::cache_err(anyhow::Error::new(err)),
            CacheOrFetchError::FetchError(err) =>
                err,
            err =>
                AuthUserProviderError::cache_err(anyhow::Error::new(err)),
        }
    }
}



pub mod mem_user_provider;

pub use mem_user_provider::InMemAuthUserProvider;
use mvv_common::backtrace::backtrace;
