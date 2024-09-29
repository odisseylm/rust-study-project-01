pub mod bits_perm_set;
pub mod hash_perm_set;
pub mod predefined;
pub mod empty_perm_provider;
pub mod util;

use core::fmt::Debug;
use core::convert::Infallible;
use core::hash::Hash;
use std::collections::HashSet;

use anyhow::anyhow;
use axum::response::Response;
use http::StatusCode;
use log::error;
use mvv_common::backtrace::{backtrace, BacktraceCell};
use crate::{AuthUserProviderError, UserId };
// -------------------------------------------------------------------------------------------------



#[derive(
    Debug,
    thiserror::Error,
    mvv_error_macro::ThisErrorFromWithBacktrace,
    mvv_error_macro::ThisErrorBacktraceSource,
)]
#[non_exhaustive]
pub enum PermissionProcessError {
    #[error("ConvertError({0})")]
    ConvertError(#[source] anyhow::Error),
    #[error("NoUser({0})")]
    NoUser(UserId, BacktraceCell),
    #[error("GetUserError({0})")]
    GetUserError(#[source] anyhow::Error),
    #[error("CacheError")]
    CacheError(#[source] anyhow::Error),
    #[error("UnknownError")]
    UnknownError(#[source] anyhow::Error),

    #[doc(hidden)]
    #[error("__NonExhaustive")]
    __NonExhaustive
}
impl PermissionProcessError {
    #[inline]
    #[track_caller]
    pub fn convert_err(err: anyhow::Error) -> Self {
        Self::ConvertError(err)
    }
    #[inline]
    #[track_caller]
    pub fn no_user_err<T>(user_id: T) -> Self where UserId: From<T> {
        Self::NoUser(user_id.into(), backtrace())
    }
    #[inline]
    #[track_caller]
    pub fn get_user_err(err: anyhow::Error) -> Self {
        Self::GetUserError(err)
    }
    #[inline]
    #[track_caller]
    pub fn cache_err(err: anyhow::Error) -> Self {
        Self::CacheError(err)
    }
    #[inline]
    #[track_caller]
    pub fn unknown_err(err: anyhow::Error) -> Self {
        Self::UnknownError(err)
    }
}


impl From<Infallible> for PermissionProcessError {
    #[inline]
    fn from(_value: Infallible) -> Self {
        PermissionProcessError::convert_err(anyhow!("Internal error: Infallible"))
    }
}

impl From<AuthUserProviderError> for PermissionProcessError {
    fn from(value: AuthUserProviderError) -> Self {
        match value {
            AuthUserProviderError::UserNotFound(user, backtrace) =>
                PermissionProcessError::NoUser(user, backtrace),
            vr @ AuthUserProviderError::DatabaseError(..) =>
                PermissionProcessError::GetUserError(vr.into()),
            vr @ AuthUserProviderError::LockedResourceError(..) =>
                PermissionProcessError::GetUserError(vr.into()),
            AuthUserProviderError::ConfigurationError(conf_err) =>
                PermissionProcessError::GetUserError(conf_err),
            AuthUserProviderError::CacheError(err) =>
                PermissionProcessError::CacheError(err),
            AuthUserProviderError::UnknownError(err) =>
                PermissionProcessError::UnknownError(err),
            //
            err => PermissionProcessError::UnknownError(anyhow::anyhow!(err)),
        }
    }
}

impl axum::response::IntoResponse for PermissionProcessError {
    fn into_response(self) -> Response {
        error!("PermissionProcess internal error: {}", &self);
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}

/// It is up to implementation use NoPermission or NoPermissions.
///
/// * In case of bits impl it is easier to return all absent bits.
/// * In case of hash-set impl it is cheaper to return NoPermission
/// with single/first missed permission without heap allocation.
///
#[derive(Debug, Clone)]
pub enum VerifyRequiredPermissionsResult <
    PermSet: PermissionSet,
> {
    RequiredPermissionsArePresent,
    /// Contains any/first absent permission
    NoPermission(<PermSet as PermissionSet>::Permission),
    /// Contains all absent permissions.
    NoPermissions(PermSet),
}
impl <
    PermSet: PermissionSet + Clone,
> VerifyRequiredPermissionsResult<PermSet> {
    #[inline(always)]
    pub fn is_authorized(&self) -> bool {
        match self {
            Self::RequiredPermissionsArePresent => true,
            _ => false,
        }
    }
}


/// Comparing HashSet it does not require heap allocation.
/// You can use bit-mask or some optimized third-party set impl.
pub trait PermissionSet : Debug + Send + Sync {
    type Permission: Hash + Eq + Debug + Clone + Send + Sync;
    fn has_permission(&self, permission: &Self::Permission) -> bool;
    fn is_empty(&self) -> bool;
    fn new() -> Self;
    fn from_permission(permission: Self::Permission) -> Self;
    fn from_permissions<const N: usize>(permissions: [Self::Permission;N]) -> Self;
    fn merge_with_mut(&mut self, another: Self);
    // ??? Use ref or values.
    fn merge(set1: Self, set2: Self) -> Self;
    // Returns missed permissions
    fn verify_required_permissions(&self, required_permissions: Self)
        -> Result<VerifyRequiredPermissionsResult<Self>, PermissionProcessError>
        where Self: Sized;
}

// It is optional trait for compatibility with axum_login::AuthzBackend
pub trait PermissionsToHashSet : Send + Sync {
    type Permission: Hash + Eq + Send + Sync;
    fn to_hash_set(&self) -> Result<HashSet<Self::Permission>, PermissionProcessError>;
}


/// It is a copy of axum_login::AuthzBackend,
/// but it does not require/depend on axum_login::AuthnBackend
/// and does not require using HashSet (with heap allocation)
// #[axum::async_trait]
#[async_trait::async_trait]
pub trait PermissionProvider: Debug + Send + Sync {
    type User: axum_login::AuthUser;
    // type Error: std::error::Error + Send + Sync;
    type Permission: Hash + Eq + Send + Sync;
    // It should be standard hash/tree set. It can be just bit mask.
    type PermissionSet: PermissionSet<Permission=Self::Permission> + Send + Sync;

    /// Gets the permissions for the provided user.
    async fn get_user_permissions(&self, user: &Self::User)
        -> Result<Self::PermissionSet, PermissionProcessError>;
    async fn get_user_permissions_by_principal_identity(
        &self, user_principal_id: <<Self as PermissionProvider>::User as axum_login::AuthUser>::Id)
        -> Result<Self::PermissionSet, PermissionProcessError>;

    /// Gets the group permissions for the provided user.
    async fn get_group_permissions(&self, user: &Self::User)
        -> Result<Self::PermissionSet, PermissionProcessError>;
    async fn get_group_permissions_by_principal_identity(
        &self, user_principal_id: <<Self as PermissionProvider>::User as axum_login::AuthUser>::Id)
        -> Result<Self::PermissionSet, PermissionProcessError>;

    /// Gets all permissions for the provided user.
    async fn get_all_permissions(&self, user: &Self::User)
        -> Result<Self::PermissionSet, PermissionProcessError> {
        let all_perms = Self::PermissionSet::merge(
            self.get_user_permissions(user).await?,
            self.get_group_permissions(user).await?
        );
        Ok(all_perms)
    }

    async fn get_all_permissions_by_principal_identity(
        &self, user_principal_id: <<Self as PermissionProvider>::User as axum_login::AuthUser>::Id)
        -> Result<Self::PermissionSet, PermissionProcessError> {
        let all_perms = Self::PermissionSet::merge(
            self.get_user_permissions_by_principal_identity(user_principal_id.clone()).await?,
            self.get_group_permissions_by_principal_identity(user_principal_id).await?
        );
        Ok(all_perms)
    }

    /// Returns a result which is `true` when the provided user has the provided
    /// permission and otherwise is `false`.
    async fn has_perm(&self, user: &Self::User, perm: Self::Permission)
        -> Result<bool, PermissionProcessError> {
        Ok(self.get_all_permissions(user).await?.has_permission(&perm))
    }
}
