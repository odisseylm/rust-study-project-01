use core::time::Duration;
use std::sync::Arc;
use log::info;
use tokio::sync::RwLock;
use mvv_auth::{
    AuthUserProvider, AuthUserProviderError,
    permission::{ PermissionSet, PermissionProcessError, PermissionProvider },
    user_provider::InMemAuthUserProvider,
};
use mvv_common::{
    cache::{ AsyncCache, TtlMode, },
};
use super::user::{AuthUser, Role, RolePermissionsSet, UserRolesExtractor };
// -------------------------------------------------------------------------------------------------


// pub type PswComparator = mvv_auth::PlainPasswordComparator;


//noinspection DuplicatedCode
#[allow(dead_code)]
pub fn in_memory_test_users()
    -> Result<InMemAuthUserProvider<AuthUser,Role,RolePermissionsSet,UserRolesExtractor>, AuthUserProviderError> {
    InMemAuthUserProvider::with_users([
        AuthUser::new(1, "vovan", "qwerty"),
        AuthUser::with_role(2, "vovan-read", "qwerty", Role::Read),
        AuthUser::with_role(3, "vovan-write", "qwerty", Role::Write),
        AuthUser::with_roles(4, "vovan-read-and-write", "qwerty",
            RolePermissionsSet::from_permissions([Role::Read, Role::Write])),
    ])
}

// We cache Option<AuthUser> to cache fact that user is not found.
// type Cache = crate::util::cache::lru::LruAsyncCache<String,Option<AuthUser>>;
// type Cache = crate::util::cache::quick_cache::QuickAsyncCache<String,Option<AuthUser>>;
type Cache = mvv_common::cache::associative_cache::AssociativeAsyncCache
                <associative_cache::Capacity128, String,Option<AuthUser>>;



#[derive(Debug)]
struct SqlUserProviderState {
    db: Arc<sqlx_postgres::PgPool>,
    cache: Option<RwLock<Cache>>,
}

#[derive(Debug)]
pub struct SqlUserProvider(Arc<SqlUserProviderState>);

//noinspection DuplicatedCode
impl SqlUserProvider {
    #[allow(dead_code)]
    pub fn new(db: Arc<sqlx_postgres::PgPool>) -> Result<SqlUserProvider, anyhow::Error> {
        Ok(SqlUserProvider(Arc::new(SqlUserProviderState { db, cache: None })))
    }
    pub fn with_cache(db: Arc<sqlx_postgres::PgPool>) -> Result<SqlUserProvider, anyhow::Error> {
        Ok(SqlUserProvider(Arc::new(SqlUserProviderState { db, cache: Some(RwLock::new(
            Cache::with_capacity_and_ttl(
                Duration::from_secs(15),
            ) ?))
        })))
    }

    #[allow(dead_code)]
    //noinspection DuplicatedCode
    async fn get_cached(&self, user_id: &String) -> Result<Option<Option<AuthUser>>,AuthUserProviderError> {
        if let Some(ref cache) = self.0.cache {
            // Can we use 'read' there?
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

        // Column 'u.name' should be case-insensitive
        let res= sqlx::query_as(
            // sqlx::query_as!(AuthUser,
            "select \
                 u.ID, u.NAME, u.PASSWORD, \
                 ur.READ_ROLE, ur.WRITE_ROLE \
                 from USERS u \
                 left join USER_ROLES ur on u.ID = ur.USER_ID \
                 where lower(u.NAME) = $1 ")
            .bind(&username)
            .fetch_optional(&*self.0.db)
            .await
            .map_err(|err_to_log|{
                log::error!("### SQLX error: {:?}", err_to_log);
                AuthUserProviderError::sqlx_err(err_to_log)
            });

        res
    }

}



#[axum::async_trait]
//noinspection DuplicatedCode
impl AuthUserProvider for SqlUserProvider {
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
//noinspection DuplicatedCode
impl PermissionProvider for SqlUserProvider {
    type User = AuthUser;
    type Permission = Role;
    type PermissionSet = RolePermissionsSet;

    async fn get_user_permissions(&self, user: &Self::User)
        -> Result<Self::PermissionSet, PermissionProcessError> {
        Ok(user.permissions())
    }

    async fn get_user_permissions_by_principal_identity(
        &self, user_principal_id: <<Self as PermissionProvider>::User as axum_login::AuthUser>::Id)
        -> Result<Self::PermissionSet, PermissionProcessError> {
        let user: Option<AuthUser> = self.get_user_by_principal_identity(&user_principal_id).await ?;
        match user {
            None => Err(PermissionProcessError::no_user_err(user_principal_id)),
            Some(ref user) => Ok(user.permissions()),
        }
    }

    async fn get_group_permissions(&self, _user: &Self::User)
        -> Result<Self::PermissionSet, PermissionProcessError> {
        Ok(RolePermissionsSet::new())
    }

    async fn get_group_permissions_by_principal_identity(
        &self, _user_principal_id: <<Self as PermissionProvider>::User as axum_login::AuthUser>::Id)
        -> Result<Self::PermissionSet, PermissionProcessError> {
        Ok(RolePermissionsSet::new())
    }
}
