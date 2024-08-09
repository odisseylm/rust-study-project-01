use core::time::Duration;
use std::sync::Arc;
use implicit_clone::ImplicitClone;
use log::info;
use tokio::sync::RwLock;
use mvv_auth::{
    AuthUserProvider, AuthUserProviderError,
    backend::OAuth2UserStore,
    permission::PermissionSet,
    // user_provider::InMemAuthUserProvider,
};
use mvv_auth::permission::{ PermissionProcessError, PermissionProvider };
use mvv_auth::util::sql::set_role_from_bool_column as set_role_from_col;
use mvv_common::backtrace::backtrace;
use mvv_common::cache::{AsyncCache, TtlMode, };
use crate::auth::{ ClientAuthUser as AuthUser, ClientFeatureSet, ClientFeature };
//--------------------------------------------------------------------------------------------------


// We cache Option<AuthUser> to cache the fact that user is not found.
// type Cache = crate::util::cache::lru::LruAsyncCache<String,Option<AuthUser>>;
// type Cache = crate::util::cache::quick_cache::QuickAsyncCache<String,Option<AuthUser>>;
type Cache = mvv_common::cache::associative_cache::AssociativeAsyncCache
                <associative_cache::Capacity128, String,Option<AuthUser>>;

#[derive(Debug)]
struct SqlClientAuthUserProviderState {
    db: Arc<sqlx_postgres::PgPool>,
    cache: Option<RwLock<Cache>>,
}

#[derive(Debug)]
pub struct SqlClientAuthUserProvider(Arc<SqlClientAuthUserProviderState>);

impl SqlClientAuthUserProvider {
    pub fn new(db: Arc<sqlx_postgres::PgPool>) -> Result<SqlClientAuthUserProvider, anyhow::Error> {
        Ok(SqlClientAuthUserProvider(Arc::new(SqlClientAuthUserProviderState { db, cache: None })))
    }
    pub fn with_cache(db: Arc<sqlx_postgres::PgPool>) -> Result<SqlClientAuthUserProvider, anyhow::Error> {
        Ok(SqlClientAuthUserProvider(Arc::new(SqlClientAuthUserProviderState { db, cache: Some(RwLock::new(
            Cache::with_capacity_and_ttl(
                Duration::from_secs(15), // nonzero_lit::u64!(15)
            ) ?))
        })))
    }

    #[allow(dead_code)]
    async fn get_cached(&self, user_id: &String) -> Result<Option<Option<AuthUser>>,AuthUserProviderError> {
        if let Some(ref cache) = self.0.cache {
            // Can we use 'read' there
            let mut cache_guarded = cache.write().await;
            let cached = (*cache_guarded).get(user_id).await ?;

            info!("### Getting user [{}] from cache (found: {})", user_id, cached.is_some());
            Ok(cached)
        } else {
            Ok(None)
        }
    }

    async fn get_user_from_db(&self, username: &str) -> Result<Option<AuthUser>, AuthUserProviderError> {

        info!("### Loading user [{}] from database", username);

        // Column 'email' should be case-insensitive in database.
        let res= sqlx::query_as(
            "select \
                 c.CLIENT_ID, c.EMAIL, cr.PSW, \
                 c.ACTIVE, c.BUSINESS_USER, c.SUPER_BUSINESS_USER \
                 from CLIENTS c \
                 inner join CLIENTS_CREDS cr on c.CLIENT_ID = cr.CLIENT_ID \
                 where lower(c.EMAIL) = $1 ")
            .bind(&username)
            .fetch_optional(&*self.0.db)
            .await
            .map_err(|err_to_log|{
                log::error!("### SQLX error: {:?}", err_to_log);
                err_to_log
            })
            // .map_err(Self::Error::Sqlx)?)
            .map_err(From::<sqlx::Error>::from);

        res
    }

}


impl sqlx::FromRow<'_, sqlx_postgres::PgRow> for crate::auth::ClientAuthUser {
    fn from_row(row: &sqlx_postgres::PgRow) -> sqlx::Result<Self> {
        use sqlx::Row;
        use mvv_common::pg_column_name as col_name;

        let client_id: uuid::Uuid = row.try_get(col_name!("CLIENT_ID")) ?;
        let email: String = row.try_get(col_name!("EMAIL") ) ?;
        let user_psw: String = row.try_get(col_name!("PSW")) ?;
        let active: bool = row.try_get(col_name!("ACTIVE")) ?;

        let mut client_features = ClientFeatureSet::new();
        if active {
            client_features.merge_with_mut(ClientFeatureSet::from_permission(ClientFeature::Standard));
            set_role_from_col(&mut client_features, ClientFeature::Business, row, col_name!("BUSINESS_USER"))?;
            set_role_from_col(&mut client_features, ClientFeature::SuperBusiness, row, col_name!("SUPER_BUSINESS_USER"))?;
        }

        Ok(crate::auth::ClientAuthUser {
            client_id: client_id.to_string(),
            email,
            active,
            password: Some(user_psw.into()),
            access_token: None,
            client_features,
        })
    }
}


#[axum::async_trait]
impl AuthUserProvider for SqlClientAuthUserProvider {
    type User = AuthUser;

    async fn get_user_by_principal_identity(&self, user_id: &<AuthUser as axum_login::AuthUser>::Id) -> Result<Option<Self::User>, AuthUserProviderError> {

        if let Some(ref cache) = self.0.cache {
            let mut cache = cache.write().await;
            // lower-case for cache key (for database it is not needed)
            let username_lc = user_id.to_lowercase();

            let cached_or_fetched = cache.get_or_fetch(username_lc, TtlMode::DefaultCacheTtl,
                |username_lc| async move { self.get_user_from_db(&username_lc).await }
            ).await ?;
            Ok(cached_or_fetched)
        } else {
            self.get_user_from_db(&user_id).await
        }
    }
}


#[axum::async_trait]
impl OAuth2UserStore for SqlClientAuthUserProvider {

    // async fn update_user_access_token22(&self, username: &String, secret_token: &str) -> Result<Option<Self::User>, AuthUserProviderError> {
    async fn update_user_access_token(&self, _user_principal_id: <<Self as AuthUserProvider>::User as axum_login::AuthUser>::Id, _secret_token: &str) -> Result<Option<Self::User>, AuthUserProviderError> {
        todo!() // TODO: impl

        /*

        // Persist user in our database, so we can use `get_user`.
        // Column 'username' should be case-insensitive.
        let user: AuthUser = sqlx::query_as(
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
        */
    }
}

// #[axum::async_trait]
#[async_trait::async_trait]
impl PermissionProvider for SqlClientAuthUserProvider {
    type User = AuthUser;
    type Permission = ClientFeature;
    type PermissionSet = ClientFeatureSet;

    async fn get_user_permissions(&self, user: &Self::User)
        -> Result<Self::PermissionSet, PermissionProcessError> {
        Ok(user.client_features.implicit_clone())
    }

    async fn get_user_permissions_by_principal_identity(
        &self, user_principal_id: <<Self as PermissionProvider>::User as axum_login::AuthUser>::Id)
        -> Result<Self::PermissionSet, PermissionProcessError> {
        let user: Option<AuthUser> = self.get_user_by_principal_identity(&user_principal_id).await ?;
        match user {
            None =>
                Err(PermissionProcessError::NoUser(user_principal_id.into(), backtrace())),
            Some(ref user) =>
                Ok(user.client_features.implicit_clone()),
        }
    }

    async fn get_group_permissions(&self, _user: &Self::User)
        -> Result<Self::PermissionSet, PermissionProcessError> {
        Ok(ClientFeatureSet::new())
    }

    async fn get_group_permissions_by_principal_identity(
        &self, _user_principal_id: <<Self as PermissionProvider>::User as axum_login::AuthUser>::Id)
        -> Result<Self::PermissionSet, PermissionProcessError> {
        Ok(ClientFeatureSet::new())
    }
}
