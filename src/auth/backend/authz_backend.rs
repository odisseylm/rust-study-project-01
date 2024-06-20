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

