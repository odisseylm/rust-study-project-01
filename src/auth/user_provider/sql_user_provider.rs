use sqlx::SqlitePool;

use super::super::{
    auth_user::AuthUser,
    auth_user_provider::{ AuthUserProvider, AuthUserProviderError },
    backend::oauth2_auth::OAuth2UserStore,
};


#[derive(Debug)]
pub struct SqlUserProvider {
    db: SqlitePool,
}


#[axum::async_trait]
impl AuthUserProvider for SqlUserProvider {
    type User = AuthUser;

    async fn get_user_by_name(&self, username: &str) -> Result<Option<Self::User>, AuthUserProviderError> {
        // TODO: use case-insensitive username comparing
        let username_lc = username.to_lowercase();
        sqlx::query_as("select * from users where lowercase(username) = ?")
            .bind(username_lc)
            .fetch_optional(&self.db)
            .await
            // .map_err(Self::Error::Sqlx)?)
            .map_err(From::<sqlx::Error>::from)
    }
    async fn get_user_by_id(&self, user_id: &<AuthUser as axum_login::AuthUser>::Id) -> Result<Option<Self::User>, AuthUserProviderError> {
        Ok(sqlx::query_as("select * from users where id = ?")
            .bind(user_id)
            .fetch_optional(&self.db)
            .await
            .map_err(AuthUserProviderError::Sqlx)
            // or
            // .map_err(From::<sqlx::Error>::from)
        ?)
    }
}


#[axum::async_trait]
impl OAuth2UserStore for SqlUserProvider {

    async fn update_user_access_token(&self, username: &str, secret_token: &str) -> Result<Option<Self::User>, AuthUserProviderError> {
        // Persist user in our database, so we can use `get_user`.
        // TODO: use case-insensitive username comparing
        let user: AuthUser = sqlx::query_as(
                r#"
                insert into users (username, access_token)
                values (?, ?)
                on conflict(username) do update
                set access_token = excluded.access_token
                returning *
                "#,
            )
            .bind(username)
            .bind(secret_token)
            .fetch_one(&self.db)
            .await
            .map_err(AuthUserProviderError::Sqlx)?;
        Ok(Some(user))
    }
}
