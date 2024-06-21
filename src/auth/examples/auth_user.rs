use core::fmt;
use crate::auth::permission::PermissionSet;
use crate::auth::permission::predefined::{ Role, RolePermissionsSet };
use crate::auth::user_provider::mem_user_provider::UserPermissionsExtractor;

use super::super::{
    backend::{ oauth2_auth::OAuth2User, psw_auth::PswUser },
    psw::PasswordComparator,
};


#[derive(Clone)]
// #[derive(serde::Serialize, serde::Deserialize)]
// #[derive(sqlx::FromRow)]
#[readonly::make]
pub struct AuthUserExample {
    pub username: String,
    pub password: Option<String>,
    pub access_token: Option<String>,
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
        user.permissions.clone()
    }
}

impl AuthUserExample {
    pub fn new(id: i64, username: &'static str, password: &'static str) -> AuthUserExample {
        AuthUserExample {
            id, username: username.to_string(), password: Some(password.to_string()),
            access_token: None,
            permissions: RolePermissionsSet::new(),
        }
    }
    pub fn access_token(&mut self, access_token: Option<String>) {
        self.access_token = access_token;
    }
    pub fn has_password<PswComparator: PasswordComparator>(&self, cred_psw: &str) -> bool {
        match self.password {
            None => false,
            Some(ref usr_psw) =>
                PswComparator::passwords_equal(usr_psw, cred_psw),
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
    fn password(&self) -> Option<String> {
        self.password.clone()
    }
    fn password_mut(&mut self, password: Option<String>) {
        self.password = password.clone()
    }
}

impl OAuth2User for AuthUserExample {
    fn access_token(&self) -> Option<String> {
        self.access_token.clone()
    }
    fn access_token_mut(&mut self, access_token: Option<String>) {
        self.access_token = access_token.clone()
    }
}


// pub fn test_users() -> Result<InMemAuthUserProvider<AuthUser>, AuthUserProviderError> {
//     InMemAuthUserProvider::<AuthUser>::with_users(vec!(AuthUser::new(1, "vovan", "qwerty")))
// }
