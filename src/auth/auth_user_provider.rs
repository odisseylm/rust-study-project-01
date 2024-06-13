use core::fmt;
use crate::auth::auth_user::AuthUser;


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

impl From<sqlx::Error> for AuthUserProviderError {
    fn from(value: sqlx::Error) -> Self {
        AuthUserProviderError::Sqlx(value)
    }
}
