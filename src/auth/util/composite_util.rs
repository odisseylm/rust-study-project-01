use std::sync::Arc;
use crate::auth::backend::authz_backend::PermissionProviderSource;
use crate::auth::permission::PermissionProvider;
use super::super::{
    error::AuthBackendError,
    backend::AuthnBackendAttributes,
    user_provider::AuthUserProvider,
};


// -------------------------------------------------------------------------------------------------
//                           Just tiny helpers to make code shorter
// -------------------------------------------------------------------------------------------------
#[inline(always)]
pub fn backend_usr_prov <
    Usr, Cred,
    B: AuthnBackendAttributes<User=Usr,Credentials=Cred,Error=AuthBackendError>,
> (backend: &Option<B>) -> Option<Arc<dyn AuthUserProvider<User=Usr> + Sync + Send>> {
    backend.as_ref().map(|b|b.user_provider())
}

#[inline(always)]
pub fn backend_usr_prov_ref <
    'a, Usr, Cred,
    B: AuthnBackendAttributes<User=Usr,Credentials=Cred,Error=AuthBackendError>,
> (backend: &'a Option<B>) -> Option<&'a Arc<dyn AuthUserProvider<User=Usr> + Sync + Send>> {
    backend.as_ref().map(|b|b.user_provider_ref())
}

#[inline(always)]
pub fn backend_perm_prov <
    Usr, Perm, PermSet,
    B: PermissionProviderSource<User=Usr,Permission=Perm,PermissionSet=PermSet>,
> (backend: &Option<B>)
   -> Option<Arc<dyn PermissionProvider<User=Usr,Permission=Perm,PermissionSet=PermSet> + Sync + Send>> {
    backend.as_ref().map(|b|{
        let as_send_sync: Arc<dyn PermissionProvider<User=Usr,Permission=Perm,PermissionSet=PermSet> + Sync + Send> = b.permission_provider();
        as_send_sync
    })
}

/*
#[inline(always)]
pub fn backend_perm_prov_ref <
    'a, Usr, Perm, PermSet,
    B: PermissionProviderSource<User=Usr,Permission=Perm,PermissionSet=PermSet>,
> (backend: &'a Option<B>)
   -> Option<&'a Arc<dyn PermissionProvider<User=Usr,Permission=Perm,PermissionSet=PermSet> + Sync + Send>> {
    backend.as_ref().map(|b|{
        let as_send_sync: Arc<dyn PermissionProvider<User=Usr,Permission=Perm,PermissionSet=PermSet> + Sync + Send> = b.permission_provider_ref();
        as_send_sync
    })
}
*/

// -------------------------------------------------------------------------------------------------


pub fn get_unique_user_provider <Usr>(
    possible_user_providers: &Vec<Option<Arc<dyn AuthUserProvider<User=Usr> + Sync + Send>>>
) -> Result<Arc<dyn AuthUserProvider<User=Usr> + Sync + Send>, AuthBackendError> {

    let all_user_providers: Vec<&Arc<dyn AuthUserProvider<User=Usr> + Sync + Send>> =
        possible_user_providers.iter().flat_map(|v|v).collect::<Vec<_>>();

    let users_provider: Arc<dyn AuthUserProvider<User=Usr> + Sync + Send> = all_user_providers
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


pub fn get_unique_permission_provider <Usr, Perm, PermSet>(
    possible_perm_providers: &Vec<Option<Arc<dyn PermissionProvider<User=Usr,Permission=Perm,PermissionSet=PermSet> + Sync + Send>>>
) -> Result<Arc<dyn PermissionProvider<User=Usr,Permission=Perm,PermissionSet=PermSet> + Sync + Send>, AuthBackendError> {

    let all_perm_providers: Vec<&Arc<dyn PermissionProvider<User=Usr,Permission=Perm,PermissionSet=PermSet> + Sync + Send>> =
        possible_perm_providers.iter().flat_map(|v|v).collect::<Vec<_>>();

    let perm_provider: Arc<dyn PermissionProvider<User=Usr,Permission=Perm,PermissionSet=PermSet> + Sync + Send> = all_perm_providers
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
