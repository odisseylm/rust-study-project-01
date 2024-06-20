use core::fmt::Debug;
use core::hash::Hash;
use core::marker::PhantomData;
use std::sync::Arc;
use crate::auth::permission::{PermissionProcessError, PermissionProvider, PermissionSet};


pub fn empty_perm_provider_arc <
    Usr: axum_login::AuthUser + 'static,
    Perm: Clone + Debug + Hash + Eq + Send + Sync + 'static,
    PermSet: PermissionSet<Permission=Perm> + Clone + Debug + Send + Sync + 'static
> () -> Arc<dyn PermissionProvider<User=Usr,Permission=Perm,PermissionSet=PermSet>> {
    Arc::new(EmptyPermProvider{ _pd: PhantomData})
}

#[derive(Debug)]
struct EmptyPermProvider <
    Usr: axum_login::AuthUser,
    Perm: Clone + Debug + Hash + Eq + Send + Sync,
    PermSet: PermissionSet<Permission=Perm> + Clone + Debug + Send + Sync
> {
    _pd: PhantomData<(Usr,Perm,PermSet)>,
}

#[async_trait::async_trait]
impl <
    Usr: axum_login::AuthUser,
    Perm: Clone + Debug + Hash + Eq + Send + Sync,
    PermSet: PermissionSet<Permission=Perm> + Clone + Debug + Send + Sync
> PermissionProvider for EmptyPermProvider<Usr,Perm,PermSet> {
    type User = Usr;
    type Permission = Perm;
    type PermissionSet = PermSet;

    async fn get_user_permissions(&self, _user: &Self::User) -> Result<Self::PermissionSet, PermissionProcessError> {
        Ok(PermissionSet::new())
    }

    async fn get_user_permissions_by_principal_identity(&self, _user_principal_id: <<Self as PermissionProvider>::User as axum_login::AuthUser>::Id) -> Result<Self::PermissionSet, PermissionProcessError> {
        Ok(PermissionSet::new())
    }

    async fn get_group_permissions(&self, _user: &Self::User) -> Result<Self::PermissionSet, PermissionProcessError> {
        Ok(PermissionSet::new())
    }

    async fn get_group_permissions_by_principal_identity(&self, _user_principal_id: <<Self as PermissionProvider>::User as axum_login::AuthUser>::Id) -> Result<Self::PermissionSet, PermissionProcessError> {
        Ok(PermissionSet::new())
    }

    async fn get_all_permissions(&self, _user: &Self::User) -> Result<Self::PermissionSet, PermissionProcessError> {
        Ok(PermissionSet::new())
    }

    async fn get_all_permissions_by_principal_identity(&self, _user_principal_id: <<Self as PermissionProvider>::User as axum_login::AuthUser>::Id) -> Result<Self::PermissionSet, PermissionProcessError> {
        Ok(PermissionSet::new())
    }

    async fn has_perm(&self, _user: &Self::User, _perm: Self::Permission) -> Result<bool, PermissionProcessError> {
        Ok(true)
    }
}