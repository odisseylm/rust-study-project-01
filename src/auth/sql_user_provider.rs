use sqlx::SqlitePool;
use crate::auth::auth_user::{AuthUser, AuthUserProvider, AuthUserProviderError};
use crate::auth::oauth2_auth::Oauth2UserProvider;


#[derive(Debug)]
pub struct SqlUserProvider {
    db: SqlitePool,
}


#[axum::async_trait]
impl AuthUserProvider for SqlUserProvider {
    type User = AuthUser;

    async fn get_user_by_name(&self, username: &str) -> Result<Option<Self::User>, AuthUserProviderError> {
        sqlx::query_as("select * from users where username = ?")
            .bind(username)
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
impl Oauth2UserProvider for SqlUserProvider {

    async fn update_user_access_token(&self, username: &str, secret_token: &str) -> Result<Option<Self::User>, AuthUserProviderError> {
        // Persist user in our database, so we can use `get_user`.
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
