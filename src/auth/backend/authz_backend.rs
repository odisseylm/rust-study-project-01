use std::collections::HashSet;
use std::convert::Infallible;
use std::hash::Hash;
use anyhow::anyhow;


trait AuthzBackend : axum_login::AuthzBackend + Clone + Send + Sync {
    type Permission: Hash + Eq + Send + Sync;
}


#[derive(thiserror::Error, Debug)]
pub enum PermissionFormatError {
    #[error("ConvertError({0})")]
    ConvertError(anyhow::Error)
}

impl From<Infallible> for PermissionFormatError {
    fn from(_value: Infallible) -> Self {
        PermissionFormatError::ConvertError(anyhow!("Internal error: Infallible"))
    }
}

/*
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


pub trait PermissionSet : Sync + Send {
    type Permission: Hash + Eq + Send + Sync;
    fn has_permission(&self, permission: &Self::Permission) -> bool;
    fn to_hash_set(&self) -> Result<HashSet<Self::Permission>, PermissionFormatError>;
    fn new() -> Self;
    fn from_permission(permission: Self::Permission) -> Self;
    fn from_permission2(perm1: Self::Permission, perm2: Self::Permission) -> Self;
    fn from_permission3(perm1: Self::Permission, perm2: Self::Permission, perm3: Self::Permission) -> Self;
    fn from_permission4(perm1: Self::Permission, perm2: Self::Permission, perm3: Self::Permission, perm4: Self::Permission) -> Self;
    fn merge_with_mut(&mut self, another: Self);
    // ??? Use ref or values.
    fn merge(set1: Self, set2: Self) -> Self;
}


/// It is a copy of axum_login::AuthzBackend, but it does not require/depend on axum_login::AuthnBackend.
trait PermissionProvider: Clone + Send + Sync {
    type User: axum_login::AuthUser;
    type Error: std::error::Error + Send + Sync;
    type Permission: Hash + Eq + Send + Sync;
    // It should be standard hash/tree set. It can be just bit mask.
    type PermissionSet: PermissionSet<Permission=Self::Permission> + Send + Sync;

    /// Gets the permissions for the provided user.
    async fn get_user_permissions(&self, user: &Self::User) -> Result<Self::PermissionSet, Self::Error>;
    async fn get_user_permissions_by_principal_identity(&self, user_principal_id: <<Self as PermissionProvider>::User as axum_login::AuthUser>::Id) -> Result<Self::PermissionSet, Self::Error>;

    /// Gets the group permissions for the provided user.
    async fn get_group_permissions(&self, user: &Self::User) -> Result<Self::PermissionSet, Self::Error>;
    async fn get_group_permissions_by_principal_identity(&self, user_principal_id: <<Self as PermissionProvider>::User as axum_login::AuthUser>::Id) -> Result<Self::PermissionSet, Self::Error>;

    /// Gets all permissions for the provided user.
    async fn get_all_permissions(&self, user: &Self::User) -> Result<Self::PermissionSet, Self::Error> {
        let all_perms = Self::PermissionSet::merge(
            self.get_user_permissions(user).await?,
            self.get_group_permissions(user).await?
        );
        Ok(all_perms)
    }

    async fn get_all_permissions_by_principal_identity(&self, user_principal_id: <<Self as PermissionProvider>::User as axum_login::AuthUser>::Id) -> Result<Self::PermissionSet, Self::Error> {
        let all_perms = Self::PermissionSet::merge(
            self.get_user_permissions_by_principal_identity(user_principal_id.clone()).await?,
            self.get_group_permissions_by_principal_identity(user_principal_id).await?
        );
        Ok(all_perms)
    }

    /// Returns a result which is `true` when the provided user has the provided
    /// permission and otherwise is `false`.
    async fn has_perm(&self, user: &Self::User, perm: Self::Permission) -> Result<bool, Self::Error> {
        Ok(self.get_all_permissions(user).await?.has_permission(&perm))
    }
}
