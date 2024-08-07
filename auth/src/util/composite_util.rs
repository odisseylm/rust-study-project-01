use std::sync::Arc;
use mvv_common::backtrace::backtrace;
use crate::{
    error::AuthBackendError,
    backend::{ AuthnBackendAttributes, authz_backend::PermissionProviderSource },
    user_provider::AuthUserProvider,
    permission::PermissionProvider,
};


/*
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
    let user_providers_count = all_user_providers.into_iter()
        .unique_by(|arc|Arc::as_ptr(arc))
        .count();
    if user_providers_count > 1 {
        return Err(AuthBackendError::DifferentUserProviders)
    }

    Ok(users_provider)
}


pub fn get_unique_permission_provider <Usr, Perm, PermSet> (
    possible_perm_providers: &Vec<Option<Arc<dyn PermissionProvider<User=Usr,Permission=Perm,PermissionSet=PermSet> + Sync + Send>>>
) -> Result<Arc<dyn PermissionProvider<User=Usr,Permission=Perm,PermissionSet=PermSet> + Sync + Send>, AuthBackendError> {

    let all_perm_providers: Vec<&Arc<dyn PermissionProvider<User=Usr,Permission=Perm,PermissionSet=PermSet> + Sync + Send>> =
        possible_perm_providers.iter().flat_map(|v|v).collect::<Vec<_>>();

    let perm_provider: Arc<dyn PermissionProvider<User=Usr,Permission=Perm,PermissionSet=PermSet> + Sync + Send> = all_perm_providers
        .first()
        .map(|arc|Arc::clone(arc))
        .ok_or_else(||AuthBackendError::NoPermissionProvider) ?;

    use itertools::Itertools;
    let perm_providers_count = all_perm_providers.into_iter()
        .unique_by(|arc|Arc::as_ptr(arc))
        .count();
    if perm_providers_count > 1 {
        return Err(AuthBackendError::DifferentPermissionProviders)
    }

    Ok(perm_provider)
}
*/


// -------------------------------------------------------------------------------------------------
//                           Just tiny helpers to make code shorter
// -------------------------------------------------------------------------------------------------


#[inline(always)]
pub fn backend_usr_prov_ref <
    'a, Usr, Cred,
    B: AuthnBackendAttributes<User=Usr,Credentials=Cred,Error=AuthBackendError>,
> (backend: &'a Option<B>) -> Option<&'a Arc<dyn AuthUserProvider<User=Usr> + Sync + Send>> {
    backend.as_ref().map(|b|b.user_provider_ref())
}

#[inline(always)]
pub fn backend_perm_prov_ref <
    'a, Usr, Perm, PermSet,
    B: PermissionProviderSource<User=Usr,Permission=Perm,PermissionSet=PermSet>,
> (backend: &'a Option<B>)
   -> Option<&'a Arc<dyn PermissionProvider<User=Usr,Permission=Perm,PermissionSet=PermSet> + Sync + Send>> {
    backend.as_ref().map(|b|{
        let as_send_sync: &Arc<dyn PermissionProvider<User=Usr,Permission=Perm,PermissionSet=PermSet> + Sync + Send> = b.permission_provider_ref();
        as_send_sync
    })
}


pub fn get_unique_user_provider_ref <'a,Usr, const N: usize>(
    possible_user_providers: [Option<&Arc<dyn AuthUserProvider<User=Usr> + Sync + Send>>;N]
) -> Result<&Arc<dyn AuthUserProvider<User=Usr> + Sync + Send>, AuthBackendError> {
    get_unique_prov_ref_impl(
        possible_user_providers,
        AuthBackendError::NoUserProvider(backtrace()),
        AuthBackendError::DifferentUserProviders(backtrace()),
    )
}

#[inline]
pub fn get_unique_permission_provider_ref <'a,Usr, Perm, PermSet, const N: usize>(
    possible_perm_providers: [Option<&Arc<dyn PermissionProvider<User=Usr,Permission=Perm,PermissionSet=PermSet> + Sync + Send>>;N]
) -> Result<&Arc<dyn PermissionProvider<User=Usr,Permission=Perm,PermissionSet=PermSet> + Sync + Send>, AuthBackendError> {
    get_unique_prov_ref_impl(
        possible_perm_providers,
        AuthBackendError::NoPermissionProvider(backtrace()),
        AuthBackendError::DifferentPermissionProviders(backtrace()),
    )
}

#[inline]
fn get_unique_prov_ref_impl <'a,Data: ?Sized, const N: usize>(
    possible_providers: [Option<&Arc<Data>>;N],
    no_provider_error: AuthBackendError,
    different_providers_error: AuthBackendError,
) -> Result<&Arc<Data>, AuthBackendError> {

    let mut first_prov: Option<&Arc<Data>> =
        possible_providers.first().and_then(|el|*el);

    for maybe_prov in possible_providers {
        match maybe_prov {
            None => {}
            Some(ref arc_ref) => {
                match first_prov {
                    None => { first_prov = Some(arc_ref); }
                    Some(first_prov) => {
                        if !Arc::ptr_eq(arc_ref, first_prov) {
                            return Err(different_providers_error)
                        }
                    }
                }
            }
        }
    }
    first_prov.ok_or_else(||no_provider_error)
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use crate::examples::auth_user::{ AuthUserExample, AuthUserExamplePswExtractor, };
    use crate::examples::composite_auth::{ Role, RolePermissionsSet, test };
    use crate::user_provider::InMemAuthUserProvider;
    use crate::test::{ TestOptionUnwrap, TestResultUnwrap, };
    use crate::util::test_unwrap::TestOps;

    type UsrProvider = dyn AuthUserProvider<User=AuthUserExample> + Send + Sync;
    type ArcUsrProvider = Arc<dyn AuthUserProvider<User=AuthUserExample> + Send + Sync>;
    type OptionArcUsrProvider<'a> = Option<&'a ArcUsrProvider>;

    #[test]
    fn test_get_unique_user_provider_ref() {

        let in_mem_users: Arc<InMemAuthUserProvider<AuthUserExample, Role, RolePermissionsSet, AuthUserExamplePswExtractor>>
            = Arc::new(test::in_memory_test_users().test_unwrap());
        let as_raw_ptr = Arc::as_ptr(&in_mem_users);

        let as_dyn_1: Arc<UsrProvider> = in_mem_users.test_clone();
        let as_dyn_2: Arc<dyn AuthUserProvider<User=AuthUserExample> + Send + Sync> = in_mem_users.test_clone();
        let as_dyn_3: Arc<dyn AuthUserProvider<User=AuthUserExample> + Send + Sync> = in_mem_users;

        let unique = get_unique_user_provider_ref([
            Some(&as_dyn_1),
            Some(&as_dyn_2),
            Some(&as_dyn_3),
        ]);
        assert_eq!(Arc::as_ptr(unique.test_unwrap()), as_raw_ptr);


        let another_in_mem_users: Arc<dyn AuthUserProvider<User=AuthUserExample> + Send + Sync> =
            Arc::new(test::in_memory_test_users().test_unwrap());
        let unique = get_unique_user_provider_ref([
            Some(&as_dyn_1),
            Some(&as_dyn_2),
            Some(&as_dyn_3),
            Some(&another_in_mem_users),
        ]);
        match unique.err().test_unwrap() {
            AuthBackendError::DifferentUserProviders(..) => {},
            _ => assert!(false, "Another error is expected"),
        }
    }

    #[test]
    fn test_get_unique_user_provider_ref_if_empty() {
        let possible_providers: [Option<&ArcUsrProvider>;0] = [];
        let unique = get_unique_user_provider_ref(possible_providers);
        match unique.err().test_unwrap() {
            AuthBackendError::NoUserProvider(..) => {},
            _ => assert!(false, "Another error is expected"),
        }

        let unique = get_unique_user_provider_ref::<OptionArcUsrProvider,1>([
            None,
        ]);
        match unique.err().test_unwrap() {
            AuthBackendError::NoUserProvider(..) => {},
            _ => assert!(false, "Another error is expected"),
        }

        let unique = get_unique_user_provider_ref(
            [None as OptionArcUsrProvider, None, None,]);
        match unique.err().test_unwrap() {
            AuthBackendError::NoUserProvider(..) => {},
            _ => assert!(false, "Another error is expected"),
        }
    }
}
