use core::fmt;


#[axum::async_trait]
pub trait AuthUserProvider : fmt::Debug {
    type User: axum_login::AuthUser;
    async fn get_user_by_principal_identity(&self, user_id: &<Self::User as axum_login::AuthUser>::Id) -> Result<Option<Self::User>, AuthUserProviderError>;
}


#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum AuthUserProviderError {
    // 1) It is used only for updates.
    // 2) If user is not found on get operation, just Ok(None) is returned.
    #[error("UserNotFound")]
    UserNotFound(String),

    #[error(transparent)]
    Sqlx(sqlx::Error),

    #[error("LockedResourceError")]
    LockedResourceError,

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

impl From<sqlx::Error> for AuthUserProviderError {
    fn from(value: sqlx::Error) -> Self {
        AuthUserProviderError::Sqlx(value)
    }
}


pub mod mem_user_provider;

pub use mem_user_provider::InMemAuthUserProvider;
