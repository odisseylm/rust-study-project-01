use core::fmt;
use anyhow::anyhow;
use mvv_auth::{
    backend::psw_auth::PswUser,
    permission::PermissionSet,
    user_provider::mem_user_provider::UserPermissionsExtractor,
};
use mvv_auth::permission::bits_perm_set::BitsPermissionSet;
use mvv_auth::permission::PermissionProcessError;
use mvv_auth::SecureString;
//--------------------------------------------------------------------------------------------------



#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, implicit_clone::ImplicitClone)]
#[derive(strum_macros::FromRepr, strum_macros::Display)]
#[repr(u32)]
#[non_exhaustive]
pub enum Role {
    // Unknown   = 0,
    Anonymous = 1 << 0,
    Read      = 1 << 1,
    Write     = 1 << 2,
}

// impl implicit_clone::ImplicitClone for Role {}
impl Into<u32> for Role {
    #[inline(always)]
    fn into(self) -> u32 { self as u32 }
}

//noinspection DuplicatedCode
impl TryFrom<u32> for Role {
    type Error = PermissionProcessError;
    #[inline]
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        let as_role: Option<Role> = Role::from_repr(value);
        as_role.ok_or_else(||Self::Error::convert_err(
            anyhow!("Conversion role error: No Role for [{}]", value)))
    }
}

// Actually it contains just roles, because it is expected there Permission=Role.
// If you need more complicated behavior, please define your own separate types
// for 'role', 'permission', and use BitsPermissionSet for permissions (or for 'role' too).
pub type RolePermissionsSet = BitsPermissionSet<u32, Role, PermissionProcessError>;


#[derive(Clone)]
#[derive(sqlx::FromRow)]
// Currently table 'USERS' // TODO: change to SOA_USERS
pub struct AuthUser {      // TODO: use diesel to avoid 2 database engines
    #[sqlx(rename = "id")]
    #[allow(dead_code)]
    pub id: i64,
    #[sqlx(rename = "name")]
    pub username: String,
    #[sqlx(rename = "password")]
    pub password: Option<SecureString>,
    #[sqlx(rename = "read_role", default)]
    pub read_role: Option<bool>,
    // pub read_role: bool,
    #[sqlx(rename = "write_role", default)]
    pub write_role: Option<bool>,
    // pub write_role: bool,
}


impl AuthUser {
    pub fn new(id: i64, username: &'static str, password: &'static str) -> AuthUser {
        AuthUser {
            id, username: username.to_string(), password: Some(password.into()),
            read_role: None, write_role: None,
        }
    }
    pub fn with_role(id: i64, username: &'static str, password: &'static str, role: Role) -> AuthUser {
        AuthUser {
            id, username: username.to_string(), password: Some(password.into()),
            read_role: if role == Role::Read { Some(true) } else { None },
            write_role: if role == Role::Write { Some(true) } else { None },
        }
    }
    pub fn with_roles(id: i64, username: &'static str, password: &'static str, roles: RolePermissionsSet) -> AuthUser {
        AuthUser {
            id, username: username.to_string(), password: Some(password.into()),
            read_role: Some(roles.has_permission(&Role::Read)),
            write_role: Some(roles.has_permission(&Role::Write)),
        }
    }
    pub fn permissions(&self) -> RolePermissionsSet {
        let mut perms = RolePermissionsSet::new();
        if self.read_role.unwrap_or(false) {
            perms.merge_with_mut(RolePermissionsSet::from_permission(Role::Read))
        }
        if self.write_role.unwrap_or(false) {
            perms.merge_with_mut(RolePermissionsSet::from_permission(Role::Write))
        }
        perms
    }
}


#[derive(Debug,Clone)]
pub struct UserRolesExtractor;
//noinspection DuplicatedCode
#[axum::async_trait]
impl UserPermissionsExtractor for UserRolesExtractor {
    type User = AuthUser;
    type Permission = Role;
    type PermissionSet = RolePermissionsSet;

    fn extract_permissions_from_user(user: &Self::User) -> Self::PermissionSet {
        user.permissions()
    }
}


//noinspection DuplicatedCode
impl fmt::Debug for AuthUser {
    //noinspection DuplicatedCode
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("User")
            .field("username", &self.username)
            .field("psw", &"[...]")
            .finish()
    }
}


//noinspection DuplicatedCode
impl axum_login::AuthUser for AuthUser {
    type Id = String;

    fn id(&self) -> Self::Id {
        self.username.to_lowercase().to_owned()
    }

    fn session_auth_hash(&self) -> &[u8] {
        unimplemented!(concat!(
            env!("CARGO_PKG_NAME"), "_AuthUser is not designed for keeping in HTTP session."))
    }
}


impl PswUser for AuthUser {
    fn password(&self) -> Option<SecureString> {
        self.password.clone()
    }
}
