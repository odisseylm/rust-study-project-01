use std::fmt::Debug;
use std::hash::Hash;
use std::sync::Arc;
use crate::auth::backend::authz_backend::PermissionProviderSource;
use crate::auth::permission::{PermissionProvider, PermissionSet};
use super::super::{
    error::AuthBackendError,
    backend::{ AuthnBackendAttributes},
    user_provider::AuthUserProvider,
};


// -------------------------------------------------------------------------------------------------
//                           Just tiny helpers to make code shorter
// -------------------------------------------------------------------------------------------------
#[inline(always)]
pub fn backend_usr_prov <
    U: axum_login::AuthUser,
    C,
    B: AuthnBackendAttributes<User=U,Credentials=C,Error=AuthBackendError>,
> (backend: &Option<B>)
    -> Option<Arc<dyn AuthUserProvider<User=U> + Sync + Send>> {
    backend.as_ref().map(|b|b.user_provider())
}

#[inline(always)]
pub fn backend_perm_prov <
    U: axum_login::AuthUser,
    P: Hash + Eq + Debug + Clone + Send + Sync,
    PS: PermissionSet<Permission=P> + Debug + Clone + Send + Sync,
    B: PermissionProviderSource<User=U,Permission=P,PermissionSet=PS>,
> (backend: &Option<B>)
   -> Option<Arc<dyn PermissionProvider<User=U,Permission=P,PermissionSet=PS> + Sync + Send>> {
    backend.as_ref().map(|b|{
        let as_send_sync: Arc<dyn PermissionProvider<User=U,Permission=P,PermissionSet=PS> + Sync + Send> = b.permission_provider().clone();
        as_send_sync
    })
}

// -------------------------------------------------------------------------------------------------


pub fn get_unique_user_provider <
    User: axum_login::AuthUser,
>(
    possible_user_providers: &Vec<Option<Arc<dyn AuthUserProvider<User=User> + Sync + Send>>>
) -> Result<Arc<dyn AuthUserProvider<User=User> + Sync + Send>, AuthBackendError> {

    let all_user_providers: Vec<&Arc<dyn AuthUserProvider<User=User> + Sync + Send>> =
        possible_user_providers.iter().flat_map(|v|v).collect::<Vec<_>>();

    let users_provider: Arc<dyn AuthUserProvider<User=User> + Sync + Send> = all_user_providers
        .first()
        .map(|arc|Arc::clone(arc))
        .ok_or_else(||AuthBackendError::NoUserProvider) ?;

    use itertools::Itertools;
    let user_providers_count = all_user_providers.into_iter().map(|arc|Arc::into_raw(Arc::clone(arc))).unique().count();
    if user_providers_count > 1 {
        return Err(AuthBackendError::DifferentUserProviders)
    }

    Ok(users_provider)
}


pub fn get_unique_permission_provider <
    User: axum_login::AuthUser,
    P: Debug + Clone + Hash + Eq + Send + Sync,
    PS: PermissionSet<Permission=P> + Debug + Clone + Send + Sync,
>(
    possible_perm_providers: &Vec<Option<Arc<dyn PermissionProvider<User=User,Permission=P,PermissionSet=PS> + Sync + Send>>>
) -> Result<Arc<dyn PermissionProvider<User=User,Permission=P,PermissionSet=PS> + Sync + Send>, AuthBackendError> {

    let all_perm_providers: Vec<&Arc<dyn PermissionProvider<User=User,Permission=P,PermissionSet=PS> + Sync + Send>> =
        possible_perm_providers.iter().flat_map(|v|v).collect::<Vec<_>>();

    let perm_provider: Arc<dyn PermissionProvider<User=User,Permission=P,PermissionSet=PS> + Sync + Send> = all_perm_providers
        .first()
        .map(|arc|Arc::clone(arc))
        .ok_or_else(||AuthBackendError::NoUserProvider) ?;

    use itertools::Itertools;
    let perm_providers_count = all_perm_providers.into_iter().map(|arc|Arc::into_raw(Arc::clone(arc))).unique().count();
    if perm_providers_count > 1 {
        return Err(AuthBackendError::DifferentUserProviders)
    }

    Ok(perm_provider)
}
