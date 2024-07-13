/*
use sqlx::SqlitePool;

use crate::{
    examples::auth_user::AuthUserExample,
    user_provider::{ AuthUserProvider, AuthUserProviderError },
    backend::oauth2_auth::OAuth2UserStore,
};


#[derive(Debug)]
pub struct SqlUserProvider {
    db: SqlitePool,
}


#[axum::async_trait]
impl AuthUserProvider for SqlUserProvider {
    type User = AuthUserExample;

    /*
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
    */
    async fn get_user_by_principal_identity(&self, user_id: &<AuthUserExample as axum_login::AuthUser>::Id) -> Result<Option<Self::User>, AuthUserProviderError> {
        // 'username' column should be case-insensitive in database
        sqlx::query_as("select * from users where username = ?")
            .bind(&username)
            .fetch_optional(&self.db)
            .await
            // .map_err(Self::Error::Sqlx)?)
            .map_err(From::<sqlx::Error>::from)
    }
}


#[axum::async_trait]
impl OAuth2UserStore for SqlUserProvider {

    // async fn update_user_access_token22(&self, username: &String, secret_token: &str) -> Result<Option<Self::User>, AuthUserProviderError> {
    async fn update_user_access_token(&self, user_principal_id: <<Self as AuthUserProvider>::User as axum_login::AuthUser>::Id, secret_token: &str) -> Result<Option<Self::User>, AuthUserProviderError> {

        // Persist user in our database, so we can use `get_user`.
        // Column 'username' should be case-insensitive
        let user: AuthUserExample = sqlx::query_as(
                r#"
                insert into users (username, access_token)
                values (?, ?)
                on conflict(username) do update
                set access_token = excluded.access_token
                returning *
                "#,
            )
            .bind(user_principal_id)
            .bind(secret_token)
            .fetch_one(&self.db)
            .await
            .map_err(AuthUserProviderError::Sqlx)?;
        Ok(Some(user))
    }
}
*/
