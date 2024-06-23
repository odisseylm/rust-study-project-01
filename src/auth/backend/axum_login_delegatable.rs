
// use axum_login::AuthnBackend;

#[cfg(feature = "ambassador")]
#[axum::async_trait]
#[ambassador::delegatable_trait_remote]
pub trait AuthnBackend: Clone + Send + Sync {
    type User: axum_login::AuthUser;
    type Credentials: Send + Sync;
    type Error: std::error::Error + Send + Sync;

    async fn authenticate(&self, creds: Self::Credentials) -> Result<Option<Self::User>, Self::Error>;
    async fn get_user(&self, user_id: &axum_login::UserId<Self>) -> Result<Option<Self::User>, Self::Error>;
}


