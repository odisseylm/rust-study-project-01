use std::sync::Arc;
use mvv_auth::{
    AuthUserProvider, AuthUserProviderError,
    backend::OAuth2UserStore,
    permission::PermissionSet,
    user_provider::InMemAuthUserProvider,
};
use mvv_auth::permission::{PermissionProcessError, PermissionProvider};
use super::user::{ AuthUser, Role, RolePermissionsSet, UserRolesExtractor };
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


#[derive(Debug)]
pub struct SqlUserProvider {
    db: Arc<sqlx_postgres::PgPool>,
}

impl SqlUserProvider {
    pub fn new(db: Arc<sqlx_postgres::PgPool>) -> Result<SqlUserProvider, anyhow::Error> {
        Ok(SqlUserProvider { db })
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
            // postgres needs lowercase
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

/*
#[inline]
fn set_role(roles: &mut RolePermissionsSet, role: Role, db_role: Option<bool>) {
    if db_role.unwrap_or(false) {
        roles.merge_with_mut(RolePermissionsSet::from_permission(role));
    }
}
*/
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
    async fn get_user_by_principal_identity(&self, user_id: &<AuthUser as axum_login::AuthUser>::Id) -> Result<Option<Self::User>, AuthUserProviderError> {
        // TODO: use case-insensitive username comparing
        let username_lc = user_id.to_lowercase();
        sqlx::query_as(
        // sqlx::query_as!(AuthUser,
            "select \
                 u.ID, u.NAME, u.PASSWORD, \
                 ur.READ_ROLE, ur.WRITE_ROLE, ur.USER_ROLE, ur.SUPER_USER_ROLE, ur.ADMIN_ROLE \
                 from USERS u \
                 left join USER_ROLES ur on u.ID = ur.USER_ID \
                 where lower(u.NAME) = $1 ")
            .bind(username_lc.as_str())
            .fetch_optional(&*self.db)
            .await
            .map_err(|err_to_log|{
                log::error!("### SQLX error: {:?}", err_to_log);
                err_to_log
            })
            // .map_err(Self::Error::Sqlx)?)
            .map_err(From::<sqlx::Error>::from)
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
