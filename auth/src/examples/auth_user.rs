use core::fmt;
use implicit_clone::ImplicitClone;
use crate::{
    SecureString,
    permission::{
        PermissionSet, predefined::{ Role, RolePermissionsSet },
    },
    user_provider::mem_user_provider::UserPermissionsExtractor,
    backend::{ oauth2_auth::OAuth2User, psw_auth::PswUser },
    psw::{ PasswordComparator },
};

#[derive(Clone)]
// #[derive(serde::Serialize, serde::Deserialize)]
// #[derive(sqlx::FromRow)]
#[readonly::make]
pub struct AuthUserExample {
    pub username: String,
    pub password: Option<SecureString>,
    pub access_token: Option<SecureString>,
    pub id: i64,
    pub permissions: RolePermissionsSet,
}

#[derive(Debug,Clone)]
pub struct AuthUserExamplePswExtractor;
#[axum::async_trait]
impl UserPermissionsExtractor for AuthUserExamplePswExtractor {
    type User = AuthUserExample;
    type Permission = Role;
    type PermissionSet = RolePermissionsSet;

    fn extract_permissions_from_user(user: &Self::User) -> Self::PermissionSet {
        user.permissions.implicit_clone()
    }
}

impl AuthUserExample {
    pub fn new(id: i64, username: &'static str, password: &'static str) -> AuthUserExample {
        Self::with_roles(id, username, password, RolePermissionsSet::new())
    }
    pub fn with_role(id: i64, username: &'static str, password: &'static str, role: Role) -> AuthUserExample {
        Self::with_roles(id, username, password, RolePermissionsSet::from_permission(role))
    }
    pub fn with_roles(id: i64, username: &'static str, password: &'static str, roles: RolePermissionsSet) -> AuthUserExample {
        AuthUserExample {
            id, username: username.to_string(), password: Some(password.into()),
            access_token: None,
            permissions: roles,
        }
    }
    pub fn access_token(&mut self, access_token: Option<SecureString>) {
        self.access_token = access_token;
    }
    pub fn has_password<PswComparator: PasswordComparator>(&self, cred_psw: &str) -> bool {
        match self.password {
            None => false,
            Some(ref usr_psw) =>
                PswComparator::passwords_equal(usr_psw.as_str(), cred_psw),
        }
    }
}


impl fmt::Debug for AuthUserExample {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("User0")
            .field("username", &self.username)
            .field("psw", &"[...]")
            .field("access_token", &"[...]")
            .finish()
    }
}

impl axum_login::AuthUser for AuthUserExample {
    type Id = String;

    fn id(&self) -> Self::Id {
        self.username.to_lowercase().clone()
    }
    fn session_auth_hash(&self) -> &[u8] {
        if let Some(access_token) = &self.access_token {
            return access_token.as_bytes();
        }

        if let Some(password) = &self.password {
            // ???
            // We use the password hash as the auth hash -> what this means
            // is when the user changes their password the auth session becomes invalid.
            //
            return password.as_bytes();
        }

        &[]
    }
}

impl PswUser for AuthUserExample {
    fn password(&self) -> Option<SecureString> {
        self.password.clone()
    }
    fn password_mut(&mut self, password: Option<SecureString>) {
        self.password = password.clone()
    }
}

impl OAuth2User for AuthUserExample {
    fn access_token(&self) -> Option<SecureString> {
        self.access_token.clone()
    }
    fn access_token_mut(&mut self, access_token: Option<SecureString>) {
        self.access_token = access_token.clone()
    }
}
