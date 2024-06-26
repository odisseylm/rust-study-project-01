use core::fmt::Debug;
use core::hash::Hash;
use std::sync::Arc;

use crate::permission::{
    PermissionProcessError, PermissionProvider, PermissionSet, VerifyRequiredPermissionsResult,
};


/// It is up to implementation use NoPermission or NoPermissions.
///
/// * In case of bits impl it is easier to return all absent bits.
/// * In case of hash-set impl it is cheaper to return NoPermission
///   with single/first missed permission without heap allocation.
///
#[derive(Debug, Clone)]
pub enum AuthorizationResult <PermSet: PermissionSet + Clone> {
    Authorized,
    /// Contains any/first absent permission.
    NoPermission(<PermSet as PermissionSet>::Permission),
    /// Contains all absent permissions.
    NoPermissions(PermSet),
}
impl <PermSet: PermissionSet + Clone> AuthorizationResult<PermSet> {
    #[inline(always)]
    pub fn is_authorized(&self) -> bool {
        match self {
            Self::Authorized => true,
            _ => false,
        }
    }
}

impl <PermSet: PermissionSet + Clone> From<VerifyRequiredPermissionsResult<PermSet>>
    for AuthorizationResult<PermSet> {

    fn from(value: VerifyRequiredPermissionsResult<PermSet>) -> Self {
        use VerifyRequiredPermissionsResult as V;
        match value {
            V::RequiredPermissionsArePresent => Self::Authorized,
            V::NoPermission(absent) => Self::NoPermission(absent),
            V::NoPermissions(absent) => Self::NoPermissions(absent),
        }
    }
}


#[cfg_attr(feature = "ambassador", ambassador::delegatable_trait)]
pub trait PermissionProviderSource {
    type User: axum_login::AuthUser;
    type Permission: Debug + Clone + Hash + Eq + Send + Sync;
    type PermissionSet: PermissionSet<Permission=Self::Permission> + Clone;

    fn permission_provider(&self) -> Arc<dyn PermissionProvider<User=Self::User,Permission=Self::Permission,PermissionSet=Self::PermissionSet> + Send + Sync>;
    fn permission_provider_ref<'a>(&'a self) -> &'a Arc<dyn PermissionProvider<User=Self::User,Permission=Self::Permission,PermissionSet=Self::PermissionSet> + Send + Sync>;
}


#[async_trait::async_trait]
#[cfg_attr(feature = "ambassador", ambassador::delegatable_trait)]
pub trait AuthorizeBackend : PermissionProviderSource + Send + Sync {

    async fn authorize(&self, user: &Self::User, required_permissions: Self::PermissionSet)
        -> Result<AuthorizationResult<Self::PermissionSet>, PermissionProcessError> {

        let user_perms = self.permission_provider().get_all_permissions(user).await ?;
        let authz_res: AuthorizationResult<Self::PermissionSet> =
            user_perms.verify_required_permissions(required_permissions) ?.into();
        Ok(authz_res)
    }

    async fn has_permission(&self, user: &Self::User, required_permission: Self::Permission)
        -> Result<bool, PermissionProcessError> {
        let authz_res: AuthorizationResult<Self::PermissionSet> = self.authorize(user,
            PermissionSet::from_permission(required_permission)).await ?;
        Ok(authz_res.is_authorized())
    }

    async fn has_permissions(&self, user: &Self::User, required_permissions: Self::PermissionSet)
        -> Result<bool, PermissionProcessError> {
        let authz_res: AuthorizationResult<Self::PermissionSet> =
            self.authorize(user, required_permissions).await ?;
        Ok(authz_res.is_authorized())
    }
}
