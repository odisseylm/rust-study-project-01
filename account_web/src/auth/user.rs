use core::fmt;
use anyhow::anyhow;
use implicit_clone::ImplicitClone;
use mvv_auth::{
    PasswordComparator,
    backend::psw_auth::PswUser,
    backend::oauth2_auth::OAuth2User,
    permission::{ PermissionSet, PermissionProcessError, bits_perm_set::BitsPermissionSet, },
    user_provider::mem_user_provider::UserPermissionsExtractor,
};
//--------------------------------------------------------------------------------------------------



#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, implicit_clone::ImplicitClone)]
#[derive(strum_macros::FromRepr, strum_macros::Display)]
#[repr(u32)]
#[non_exhaustive]
pub enum ClientType {
    // Unknown     = 0,
    Usual          = 1 << 0,
    Business       = 1 << 1,
    SuperBusiness  = 1 << 2,
}

impl Into<u32> for ClientType {
    #[inline(always)]
    fn into(self) -> u32 { self as u32 }
}
impl TryFrom<u32> for ClientType {
    type Error = PermissionProcessError;
    #[inline]
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        let as_role: Option<ClientType> = ClientType::from_repr(value);
        as_role.ok_or_else(||PermissionProcessError::ConvertError(
            anyhow!("Conversion role error: No ClientType for [{}]", value)))
    }
}

pub type Role = ClientType;
pub type RolePermissionsSet = BitsPermissionSet<u32, ClientType, PermissionProcessError>;


#[derive(Clone)]
pub struct ClientAuthUser {
    pub client_id: String,
    pub username: String,
    pub password: Option<String>,
    pub access_token: Option<String>,
    pub client_features: RolePermissionsSet,
}


impl ClientAuthUser {
    pub fn test_std_client(client_id: &'static str, username: &'static str, password: &'static str) -> ClientAuthUser {
        ClientAuthUser {
            client_id: client_id.to_string(), username: username.to_string(),
            password: Some(password.to_string()),
            access_token: None,
            client_features: RolePermissionsSet::from_permission(ClientType::Usual),
        }
    }
    pub fn test_client_with_type(client_id: &'static str, username: &'static str, password: &'static str, client_type: ClientType) -> ClientAuthUser {
        ClientAuthUser {
            client_id: client_id.to_string(), username: username.to_string(),
            password: Some(password.to_string()),
            access_token: None,
            client_features: RolePermissionsSet::from_permissions([ClientType::Usual, client_type]),
        }
    }
    pub fn test_client_with_features(client_id: &'static str, username: &'static str, password: &'static str, client_features: RolePermissionsSet) -> ClientAuthUser {
        ClientAuthUser {
            client_id: client_id.to_string(), username: username.to_string(),
            password: Some(password.to_string()),
            access_token: None,
            client_features,
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


#[derive(Debug,Clone)]
pub struct ClientFeaturesExtractor;
#[axum::async_trait]
impl UserPermissionsExtractor for ClientFeaturesExtractor {
    type User = ClientAuthUser;
    type Permission = Role;
    type PermissionSet = RolePermissionsSet;

    fn extract_permissions_from_user(user: &Self::User) -> Self::PermissionSet {
        user.client_features.implicit_clone()
    }
}


impl fmt::Debug for ClientAuthUser {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Client")
            .field("username", &self.username)
            .field("features", &self.client_features)
            .field("psw", &"[...]")
            .field("access_token", &"[...]")
            .finish()
    }
}


impl axum_login::AuthUser for ClientAuthUser {
    type Id = String;

    fn id(&self) -> Self::Id {
        self.username.to_lowercase().to_owned()
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


impl PswUser for ClientAuthUser {
    fn password(&self) -> Option<String> {
        self.password.clone()
    }
    fn password_mut(&mut self, password: Option<String>) {
        self.password = password.clone()
    }
}


impl OAuth2User for ClientAuthUser {
    fn access_token(&self) -> Option<String> {
        self.access_token.clone()
    }
    fn access_token_mut(&mut self, access_token: Option<String>) {
        self.access_token = access_token.clone()
    }
}
