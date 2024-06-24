use core::fmt::Debug;
use core::hash::Hash;
use core::marker::PhantomData;
use std::sync::Arc;
use crate::auth::permission::{PermissionProcessError, PermissionProvider, PermissionSet, VerifyRequiredPermissionsResult};


pub fn always_allowed_perm_provider_arc <
    User: axum_login::AuthUser + 'static,
    Perm: Clone + Debug + Hash + Eq + Send + Sync + 'static,
    PermSet: PermissionSet<Permission=Perm> + Clone + 'static,
> () -> Arc<dyn PermissionProvider<User=User,Permission=Perm,PermissionSet=PermSet>> {
    Arc::new(EmptyPermProvider::<User,Perm,PermSet,true> { _pd: PhantomData})
}
pub fn always_denied_perm_provider_arc <
    User: axum_login::AuthUser + 'static,
    Perm: Clone + Debug + Hash + Eq + Send + Sync + 'static,
    PermSet: PermissionSet<Permission=Perm> + Clone + 'static,
> () -> Arc<dyn PermissionProvider<User=User,Permission=Perm,PermissionSet=PermSet>> {
    Arc::new(EmptyPermProvider::<User,Perm,PermSet,false> { _pd: PhantomData})
}


pub fn empty_always_allowed_perm_provider_arc <
    User: axum_login::AuthUser + 'static,
    Perm: Clone + Debug + Hash + Eq + Send + Sync + 'static,
> () -> Arc<dyn PermissionProvider<User=User,Permission=Perm,PermissionSet=AlwaysAllowedPermSet<Perm>>> {
    Arc::new(EmptyPermProvider::<User,Perm,AlwaysAllowedPermSet<Perm>,true> { _pd: PhantomData})
}
pub fn empty_always_denied_perm_provider_arc <
    User: axum_login::AuthUser + 'static,
    Perm: Clone + Debug + Hash + Eq + Send + Sync + 'static,
> () -> Arc<dyn PermissionProvider<User=User,Permission=Perm,PermissionSet=AlwaysDeniedPermSet<Perm>>> {
    Arc::new(EmptyPermProvider::<User,Perm,AlwaysDeniedPermSet<Perm>,false> { _pd: PhantomData})
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct EmptyPerm;

#[derive(Debug)]
struct EmptyPermProvider <
    User: axum_login::AuthUser,
    Perm: Clone + Debug + Hash + Eq + Send + Sync,// = EmptyPerm,
    PermSet: PermissionSet<Permission=Perm> + Clone,
    const ALLOWED: bool,
> {
    _pd: PhantomData<(User, Perm, PermSet)>,
}

#[async_trait::async_trait]
impl <
    Usr: axum_login::AuthUser,
    Perm: Clone + Debug + Hash + Eq + Send + Sync,
    PermSet: PermissionSet<Permission=Perm> + Clone,
    const ALLOWED: bool,
> PermissionProvider for EmptyPermProvider<Usr,Perm,PermSet,ALLOWED> {
    type User = Usr;
    type Permission = Perm;
    type PermissionSet = PermSet;

    async fn get_user_permissions(&self, _user: &Self::User)
        -> Result<Self::PermissionSet, PermissionProcessError> {
        Ok(PermissionSet::new())
    }

    async fn get_user_permissions_by_principal_identity(
        &self, _user_principal_id: <<Self as PermissionProvider>::User as axum_login::AuthUser>::Id)
        -> Result<Self::PermissionSet, PermissionProcessError> {
        Ok(PermissionSet::new())
    }

    //noinspection DuplicatedCode
    async fn get_group_permissions(&self, _user: &Self::User)
        -> Result<Self::PermissionSet, PermissionProcessError> {
        Ok(PermissionSet::new())
    }

    //noinspection DuplicatedCode
    async fn get_group_permissions_by_principal_identity(
        &self, _user_principal_id: <<Self as PermissionProvider>::User as axum_login::AuthUser>::Id)
        -> Result<Self::PermissionSet, PermissionProcessError> {
        Ok(PermissionSet::new())
    }

    async fn get_all_permissions(&self, _user: &Self::User)
        -> Result<Self::PermissionSet, PermissionProcessError> {
        Ok(PermissionSet::new())
    }

    async fn get_all_permissions_by_principal_identity(
        &self, _user_principal_id: <<Self as PermissionProvider>::User as axum_login::AuthUser>::Id)
        -> Result<Self::PermissionSet, PermissionProcessError> {
        Ok(PermissionSet::new())
    }

    async fn has_perm(&self, _user: &Self::User, _perm: Self::Permission)
        -> Result<bool, PermissionProcessError> {
        Ok(ALLOWED)
    }
}


#[derive(Debug, Clone)]
pub struct AlwaysAllowedPermSet <Perm> {
    _pd: PhantomData<Perm>,
}
impl <
    Perm: Hash + Eq + Debug + Clone + Send + Sync,
> PermissionSet for AlwaysAllowedPermSet<Perm> {
    type Permission = Perm;

    #[inline(always)]
    fn has_permission(&self, _permission: &Self::Permission) -> bool {
        true
    }
    #[inline(always)]
    fn is_empty(&self) -> bool {
        true
    }
    #[inline(always)]
    fn new() -> Self {
        AlwaysAllowedPermSet::<Perm> { _pd: PhantomData }
    }
    #[inline(always)]
    fn from_permission(_permission: Self::Permission) -> Self {
        Self::new()
    }
    #[inline(always)]
    fn from_permissions<const N: usize>(_permissions: [Self::Permission; N]) -> Self {
        Self::new()
    }
    /*
    #[inline(always)]
    fn from_permission2(_perm1: Self::Permission, _perm2: Self::Permission) -> Self {
        Self::new()
    }
    #[inline(always)]
    fn from_permission3(_perm1: Self::Permission, _perm2: Self::Permission, _perm3: Self::Permission) -> Self {
        Self::new()
    }
    #[inline(always)]
    fn from_permission4(_perm1: Self::Permission, _perm2: Self::Permission, _perm3: Self::Permission, _perm4: Self::Permission) -> Self {
        Self::new()
    }
    */
    #[inline(always)]
    fn merge_with_mut(&mut self, _another: Self) {
    }
    #[inline(always)]
    fn merge(_set1: Self, _set2: Self) -> Self {
        Self::new()
    }
    #[inline(always)]
    fn verify_required_permissions(&self, _required_permissions: Self) -> Result<VerifyRequiredPermissionsResult<Self::Permission, Self>, PermissionProcessError> where Self: Sized {
        Ok(VerifyRequiredPermissionsResult::RequiredPermissionsArePresent)
    }
}


#[derive(Debug, Clone)]
pub struct AlwaysDeniedPermSet <Perm> {
    _pd: PhantomData<Perm>,
}
impl <
    Perm: Hash + Eq + Debug + Clone + Send + Sync,
> PermissionSet for AlwaysDeniedPermSet<Perm> {
    type Permission = Perm;

    #[inline(always)]
    fn has_permission(&self, _permission: &Self::Permission) -> bool {
        false
    }
    #[inline(always)]
    fn is_empty(&self) -> bool {
        true
    }
    #[inline(always)]
    fn new() -> Self {
        AlwaysDeniedPermSet::<Perm> { _pd: PhantomData }
    }
    #[inline(always)]
    fn from_permission(_permission: Self::Permission) -> Self {
        Self::new()
    }
    #[inline(always)]
    fn from_permissions<const N: usize>(_permissions: [Self::Permission; N]) -> Self {
        Self::new()
    }
    /*
    #[inline(always)]
    fn from_permission2(_perm1: Self::Permission, _perm2: Self::Permission) -> Self {
        Self::new()
    }
    #[inline(always)]
    fn from_permission3(_perm1: Self::Permission, _perm2: Self::Permission, _perm3: Self::Permission) -> Self {
        Self::new()
    }
    #[inline(always)]
    fn from_permission4(_perm1: Self::Permission, _perm2: Self::Permission, _perm3: Self::Permission, _perm4: Self::Permission) -> Self {
        Self::new()
    }
    */
    #[inline(always)]
    fn merge_with_mut(&mut self, _another: Self) {
    }
    #[inline(always)]
    fn merge(_set1: Self, _set2: Self) -> Self {
        Self::new()
    }
    #[inline(always)]
    fn verify_required_permissions(&self, _required_permissions: Self) -> Result<VerifyRequiredPermissionsResult<Self::Permission, Self>, PermissionProcessError> where Self: Sized {
        Ok(VerifyRequiredPermissionsResult::NoPermissions(Self::new()))
    }
}
