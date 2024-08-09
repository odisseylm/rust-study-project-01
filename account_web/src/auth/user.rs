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
use mvv_auth::SecureString;
//--------------------------------------------------------------------------------------------------


/// It is used there as role/permission to allow/deny views/action.
///
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, implicit_clone::ImplicitClone)]
#[derive(strum_macros::FromRepr, strum_macros::Display)]
#[repr(u32)]
#[non_exhaustive]
pub enum ClientFeature {
    // Unknown     = 0,
    Standard       = 1 << 0,
    Business       = 1 << 1,
    SuperBusiness  = 1 << 2,
}

impl Into<u32> for ClientFeature {
    #[inline(always)]
    fn into(self) -> u32 { self as u32 }
}
impl TryFrom<u32> for ClientFeature {
    type Error = PermissionProcessError;
    #[inline]
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        let as_role: Option<ClientFeature> = ClientFeature::from_repr(value);
        as_role.ok_or_else(||PermissionProcessError::ConvertError(
            anyhow!("Conversion role error: No ClientType for [{}]", value)))
    }
}

pub type Role = ClientFeature;
pub type RolePermissionsSet = BitsPermissionSet<u32, ClientFeature, PermissionProcessError>;
pub type ClientFeatureSet = BitsPermissionSet<u32, ClientFeature, PermissionProcessError>;


#[derive(Clone)]
pub struct ClientAuthUser {
    pub client_id: String,
    pub email: String,
    pub active: bool,
    pub password: Option<SecureString>,
    pub access_token: Option<SecureString>,
    pub client_features: RolePermissionsSet,
}


impl ClientAuthUser {
    pub fn test_std_client(client_id: &'static str, email: &'static str, password: &'static str) -> ClientAuthUser {
        ClientAuthUser {
            client_id: client_id.to_string(), email: email.to_string(),
            active: true,
            password: Some(password.into()),
            access_token: None,
            client_features: RolePermissionsSet::from_permission(ClientFeature::Standard),
        }
    }
    pub fn test_client_with_type(
        client_id: &'static str, email: &'static str, password: &'static str,
        client_feature: ClientFeature) -> ClientAuthUser {
        ClientAuthUser {
            client_id: client_id.to_string(), email: email.to_string(),
            active: true,
            password: Some(password.into()),
            access_token: None,
            client_features: RolePermissionsSet::from_permissions([ClientFeature::Standard, client_feature]),
        }
    }
    pub fn test_client_with_features(client_id: &'static str, email: &'static str, password: &'static str, client_features: RolePermissionsSet) -> ClientAuthUser {
        ClientAuthUser {
            client_id: client_id.to_string(), email: email.to_string(),
            active: true,
            password: Some(password.into()),
            access_token: None,
            client_features,
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
            .field("email", &self.email)
            .field("features", &self.client_features)
            .field("psw", &"[...]")
            .field("access_token", &"[...]")
            .finish()
    }
}


impl axum_login::AuthUser for ClientAuthUser {
    type Id = String;

    fn id(&self) -> Self::Id {
        self.email.to_lowercase().to_owned()
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
    fn password(&self) -> Option<SecureString> {
        self.password.clone()
    }
    fn password_mut(&mut self, password: Option<SecureString>) {
        self.password = password.clone()
    }
}


impl OAuth2User for ClientAuthUser {
    fn access_token(&self) -> Option<SecureString> {
        self.access_token.clone()
    }
    fn access_token_mut(&mut self, access_token: Option<SecureString>) {
        self.access_token = access_token.clone()
    }
}
