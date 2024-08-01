pub mod http;
pub mod composite_util;
mod backend_dyn_wrap;// pub(crate) mod temp;
mod user_provider_wrap;
mod backend_delegate;
pub mod fmt;
pub mod test_unwrap;


pub mod sql {
    use crate::permission::PermissionSet;

    pub fn set_role_from_bool_column<'r,
        DbRow: sqlx::Row,
        Perm, PermSet: PermissionSet<Permission = Perm>,
        I,
    > (roles: &mut PermSet, role: Perm, row: &'r DbRow, column: I)
    -> Result<(), sqlx::Error>
    where
        I: sqlx::ColumnIndex<DbRow>,
        bool: sqlx::Decode<'r, DbRow::Database> + sqlx::Type<DbRow::Database>,
    {
        let db_role: Option<bool> = row.try_get::<'r, Option<bool>, I>(column) ?;
        if db_role.unwrap_or(false) {
            roles.merge_with_mut(PermSet::from_permission(role));
        }
        Ok(())
    }

}