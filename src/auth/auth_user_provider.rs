use core::fmt;


#[axum::async_trait]
pub trait AuthUserProvider : fmt::Debug {
    type User: axum_login::AuthUser;
    // TODO: rename to 'get_user_by_principal_identity'
    async fn get_user_by_id(&self, user_id: &<Self::User as axum_login::AuthUser>::Id) -> Result<Option<Self::User>, AuthUserProviderError>;
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


