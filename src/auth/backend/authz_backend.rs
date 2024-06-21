use core::fmt::Debug;

/*

use std::collections::HashSet;
use std::convert::Infallible;
use std::hash::Hash;
use anyhow::anyhow;

trait AuthzBackend : axum_login::AuthzBackend + Clone + Send + Sync {
    type Permission: Hash + Eq + Send + Sync;
}


/// It is a copy of axum_login::AuthzBackend, but it does not require/depend on axum_login::AuthnBackend.
trait AxumLoginPermissionProvider: Clone + Send + Sync {
    type User: axum_login::AuthUser;
    type Error: std::error::Error + Send + Sync;
    type Permission: Hash + Eq + Send + Sync;
    // It should be standard hash/tree set. It can be just bit mask.
    type PermissionSet: Hash + Eq + Send + Sync;

    /// Gets the permissions for the provided user.
    async fn get_user_permissions(&self, user: &Self::User) -> Result<HashSet<Self::Permission>, Self::Error>;

    /// Gets the group permissions for the provided user.
    async fn get_group_permissions(&self, user: &Self::User) -> Result<HashSet<Self::Permission>, Self::Error>;

    /// Gets all permissions for the provided user.
    async fn get_all_permissions(&self, user: &Self::User) -> Result<HashSet<Self::Permission>, Self::Error> {
        let mut all_perms = HashSet::new();
        all_perms.extend(self.get_user_permissions(user).await?);
        all_perms.extend(self.get_group_permissions(user).await?);
        Ok(all_perms)
    }

    /// Returns a result which is `true` when the provided user has the provided
    /// permission and otherwise is `false`.
    async fn has_perm(&self, user: &Self::User, perm: Self::Permission) -> Result<bool, Self::Error> {
        Ok(self.get_all_permissions(user).await?.contains(&perm))
    }
}
*/

use std::hash::Hash;
use std::sync::Arc;
use crate::auth::permission::{PermissionProcessError, PermissionProvider, PermissionSet, VerifyRequiredPermissionsResult};



/// It is up to implementation use NoPermission or NoPermissions.
///
/// * In case of bits impl it is easier to return all absent bits.
/// * In case of hash-set impl it is cheaper to return NoPermission
/// with single/first missed permission without heap allocation.
///
#[derive(Debug, Clone)]
pub enum AuthorizationResult <
    Perm: Debug + Clone + Hash + Eq + Send + Sync,
    PermSet: PermissionSet<Permission=Perm> + Debug + Clone + Send + Sync,
> {
    Authorized,

    /// Contains any/first absent permission
    NoPermission(Perm),
    /// Contains all absent permissions.
    NoPermissions(PermSet),
}
impl <
    Perm: Debug + Clone + Hash + Eq + Send + Sync,
    PermSet: PermissionSet<Permission=Perm> + Debug + Clone + Send + Sync
> AuthorizationResult<Perm,PermSet> {
    #[inline(always)]
    pub fn is_authorized(&self) -> bool {
        match self {
            Self::Authorized => true,
            _ => false,
        }
    }
}

impl <
    Perm: Debug + Clone + Hash + Eq + Send + Sync,
    PermSet: PermissionSet<Permission=Perm> + Debug + Clone + Send + Sync
> From<VerifyRequiredPermissionsResult<Perm,PermSet>> for AuthorizationResult<Perm,PermSet> {
    fn from(value: VerifyRequiredPermissionsResult<Perm, PermSet>) -> Self {
        use VerifyRequiredPermissionsResult as V;
        match value {
            V::RequiredPermissionsArePresent => Self::Authorized,
            V::NoPermission(absent) => Self::NoPermission(absent),
            V::NoPermissions(absent) => Self::NoPermissions(absent),
        }
    }
}



pub trait PermissionProviderSource : Clone + Send + Sync {
    type User: axum_login::AuthUser;
    type Permission: Debug + Clone + Hash + Eq + Send + Sync;
    type PermissionSet: PermissionSet<Permission=Self::Permission> + Debug + Clone + Send + Sync;

    fn permission_provider(&self) -> Arc<dyn PermissionProvider<User=Self::User,Permission=Self::Permission,PermissionSet=Self::PermissionSet>>;
    // fn permission_provider_ref(&self) -> &Arc<dyn PermissionProvider<User=Self::User,Permission=Self::Permission,PermissionSet=Self::PermissionSet>>;
}


#[async_trait::async_trait]
pub trait AuthorizeBackend : PermissionProviderSource + Clone + Send + Sync {
    // type User: axum_login::AuthUser;
    // type Permission: Debug + Clone + Hash + Eq + Send + Sync;
    // type PermissionSet: PermissionSet<Permission=Self::Permission> + Debug + Clone + Send + Sync;

    // async fn authorize(&self, user: &Self::User, permissions: Self::PermissionSet) -> Result<AuthorizationResult<Self::Permission,Self::PermissionSet>,PermissionProcessError>;
    // async fn has_permission(&self, user: &Self::User, permissions: Self::Permission) -> Result<bool, PermissionProcessError>;
    // async fn has_permissions(&self, user: &Self::User, permissions: Self::PermissionSet) -> Result<bool, PermissionProcessError>;

    async fn authorize(&self, user: &Self::User, required_permissions: Self::PermissionSet)
        -> Result<AuthorizationResult<Self::Permission,Self::PermissionSet>, PermissionProcessError> {

        let user_perms = self.permission_provider().get_all_permissions(user).await ?;
        let authz_res: AuthorizationResult<Self::Permission,Self::PermissionSet> =
            user_perms.verify_required_permissions(required_permissions) ?.into();
        Ok(authz_res)
    }

    async fn has_permission(&self, user: &Self::User, required_permission: Self::Permission) -> Result<bool, PermissionProcessError> {
        let authz_res: AuthorizationResult<Self::Permission,Self::PermissionSet> = self.authorize(user,
            PermissionSet::from_permission(required_permission)).await ?;
        Ok(authz_res.is_authorized())
    }

    async fn has_permissions(&self, user: &Self::User, required_permissions: Self::PermissionSet) -> Result<bool, PermissionProcessError> {
        let authz_res: AuthorizationResult<Self::Permission,Self::PermissionSet> =
            self.authorize(user, required_permissions).await ?;
        Ok(authz_res.is_authorized())
    }
}
