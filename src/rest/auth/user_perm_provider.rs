use core::time::Duration;
use std::sync::Arc;
use log::info;
use tokio::sync::RwLock;
use mvv_auth::{
    AuthUserProvider, AuthUserProviderError,
    backend::OAuth2UserStore,
    permission::PermissionSet,
    user_provider::InMemAuthUserProvider,
};
use mvv_auth::permission::{PermissionProcessError, PermissionProvider};
use super::user::{ AuthUser, Role, RolePermissionsSet, UserRolesExtractor };
use crate::util::cache::{AsyncCache, CacheOrFetchError, lru};
// -------------------------------------------------------------------------------------------------


pub type PswComparator = mvv_auth::PlainPasswordComparator;


pub fn in_memory_test_users()
    -> Result<InMemAuthUserProvider<AuthUser,Role,RolePermissionsSet,UserRolesExtractor>, AuthUserProviderError> {
    InMemAuthUserProvider::with_users(vec!(
        AuthUser::new(1, "vovan", "qwerty"),
        AuthUser::with_role(1, "vovan-read", "qwerty", Role::Read),
        AuthUser::with_role(1, "vovan-write", "qwerty", Role::Write),
        AuthUser::with_roles(1, "vovan-read-and-write", "qwerty",
            RolePermissionsSet::from_permissions([Role::Read, Role::Write])),
    ))
}

// We cache Option<AuthUser> to cache fact that user is not found.
type Cache = lru::LruAsyncCache<String,Option<AuthUser>>;


impl From<CacheOrFetchError<AuthUserProviderError>> for AuthUserProviderError {
    fn from(value: CacheOrFetchError<AuthUserProviderError>) -> Self {
        match value {
            CacheOrFetchError::CacheError(err) =>
                AuthUserProviderError::CacheError(anyhow::Error::new(err)),
            CacheOrFetchError::FetchError(err) => err,
        }
    }
}


#[derive(Debug)]
struct SqlUserProviderState {
    db: Arc<sqlx_postgres::PgPool>,
    cache: Option<RwLock<Cache>>,
}

#[derive(Debug)]
pub struct SqlUserProvider(Arc<SqlUserProviderState>);

impl SqlUserProvider {
    pub fn new(db: Arc<sqlx_postgres::PgPool>) -> Result<SqlUserProvider, anyhow::Error> {
        Ok(SqlUserProvider(Arc::new(SqlUserProviderState { db, cache: None })))
    }
    pub fn with_cache(db: Arc<sqlx_postgres::PgPool>) -> Result<SqlUserProvider, anyhow::Error> {
        Ok(SqlUserProvider(Arc::new(SqlUserProviderState { db, cache: Some(RwLock::new(
            Cache::with_capacity_and_ttl(
                nonzero_lit::usize!(100),
                Duration::from_secs(15),
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

    #[allow(dead_code)]
    async fn update_cache(&self, user_id: String, user: Option<AuthUser>)
        -> Result<(),AuthUserProviderError> {
        if let Some(ref cache) = self.0.cache {
            let mut cache_guarded = cache.write().await;
            (*cache_guarded).put_with_ttl(user_id, user).await ?;
        }
        Ok(())
    }

    async fn get_user_from_db(&self, username: &str) -> Result<Option<AuthUser>, AuthUserProviderError> {

        let username_lc = username.to_lowercase();

        info!("### Loading user [{}] from database", username_lc);

        // TODO: use case-insensitive username comparing in SQL query
        let res= sqlx::query_as(
            // sqlx::query_as!(AuthUser,
            "select \
                 u.ID, u.NAME, u.PASSWORD, \
                 ur.READ_ROLE, ur.WRITE_ROLE, ur.USER_ROLE, ur.SUPER_USER_ROLE, ur.ADMIN_ROLE \
                 from USERS u \
                 left join USER_ROLES ur on u.ID = ur.USER_ID \
                 where lower(u.NAME) = $1 ")
            .bind(username_lc.as_str())
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


// impl<'r, R> sqlx::FromRow<'r, R> for AuthUser where R: sqlx::Row {
//     fn from_row(row: &'r R) -> Result<Self, sqlx::Error> {
// impl sqlx::FromRow<'_, sqlx::any::AnyRow> for AuthUser {
//     fn from_row(row: &sqlx::any::AnyRow) -> sqlx::Result<Self> {
impl sqlx::FromRow<'_, sqlx_postgres::PgRow> for AuthUser {
    fn from_row(row: &sqlx_postgres::PgRow) -> sqlx::Result<Self> {

        use sqlx::Row;
        macro_rules! column_name {
            // postgres needs lowercase (Oracle - uppercase, so on)
            ($column_name:literal) => { const_str::convert_ascii_case!(lower, $column_name) };
        }

        let user_id: i64 = row.try_get(column_name!("ID")) ?;
        let username: String = row.try_get(column_name!("NAME") ) ?;
        let user_psw: String = row.try_get(column_name!("password")) ?;

        let mut roles = RolePermissionsSet::new();
        set_role(&mut roles, Role::Read, &row, column_name!("read_role")) ?;
        set_role(&mut roles, Role::Write, &row, column_name!("write_role")) ?;
        set_role(&mut roles, Role::Write, &row, column_name!("user_role")) ?;
        set_role(&mut roles, Role::SuperUser, &row, column_name!("super_user_role")) ?;
        set_role(&mut roles, Role::Admin, &row, column_name!("admin_role")) ?;

        Ok(AuthUser {
            id: user_id,
            username,
            password: Some(user_psw),
            access_token: None,
            permissions: roles,
        })
    }
}

#[inline]
fn set_role(roles: &mut RolePermissionsSet, role: Role, row: &sqlx_postgres::PgRow, column: &'static str)
    -> Result<(), sqlx::Error> {
    use sqlx::Row;
    let db_role: Option<bool> = row.try_get(column) ?;
    if db_role.unwrap_or(false) {
        roles.merge_with_mut(RolePermissionsSet::from_permission(role));
    }
    Ok(())
}


#[axum::async_trait]
impl AuthUserProvider for SqlUserProvider {
    type User = AuthUser;

    async fn get_user_by_principal_identity(&self, user_id: &<AuthUser as axum_login::AuthUser>::Id) -> Result<Option<Self::User>, AuthUserProviderError> {

        let username_lc = user_id.to_lowercase();

        if let Some(ref cache) = self.0.cache {
            let mut cache = cache.write().await;

            let cached_or_fetched = cache.get_or_fetch(username_lc, |username_lc| async move {
                self.get_user_from_db(&username_lc).await
            }).await ?;
            Ok(cached_or_fetched)
        } else {
            self.get_user_from_db(username_lc.as_str()).await
        }

        /*
        if let Some(cached_user) = self.get_cached(&username_lc).await ? {
            return Ok(cached_user);
        }

        let user_opt = self.get_user_from_db(user_id).await ?;

        self.update_cache(username_lc, user_opt.clone()).await ?;
        Ok(user_opt)
        */
    }
}


#[axum::async_trait]
impl OAuth2UserStore for SqlUserProvider {

    // async fn update_user_access_token22(&self, username: &String, secret_token: &str) -> Result<Option<Self::User>, AuthUserProviderError> {
    async fn update_user_access_token(&self, _user_principal_id: <<Self as AuthUserProvider>::User as axum_login::AuthUser>::Id, _secret_token: &str) -> Result<Option<Self::User>, AuthUserProviderError> {
        todo!()

        /*
        let user_principal_id = user_principal_id.to_lowercase();

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
impl PermissionProvider for SqlUserProvider {
    type User = AuthUser;
    type Permission = Role;
    type PermissionSet = RolePermissionsSet;

    async fn get_user_permissions(&self, user: &Self::User)
        -> Result<Self::PermissionSet, PermissionProcessError> {
        Ok(user.permissions.clone())
    }

    async fn get_user_permissions_by_principal_identity(
        &self, user_principal_id: <<Self as PermissionProvider>::User as axum_login::AuthUser>::Id)
        -> Result<Self::PermissionSet, PermissionProcessError> {
        let user: Option<AuthUser> = self.get_user_by_principal_identity(&user_principal_id).await ?;
        match user {
            None => Err(PermissionProcessError::NoUser(user_principal_id.to_string())),
            Some(ref user) => Ok(user.permissions.clone()),
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
