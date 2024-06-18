#![allow(dead_code)]

use std::sync::Arc;
use axum::extract::Request;
use axum::response::{ IntoResponse, Response };
use super::super::{
    error::AuthBackendError,
    auth_backend::{ AuthnBackendAttributes},
    auth_user_provider::AuthUserProvider,
    auth_user::AuthUser,
};


#[inline(always)]
pub fn get_user_provider2 <
    C1, C2,
    B1: AuthnBackendAttributes<User=AuthUser,Credentials=C1,Error=AuthBackendError>,
    B2: AuthnBackendAttributes<User=AuthUser,Credentials=C2,Error=AuthBackendError>,
>(
    backend1: &Option<B1>,
    backend2: &Option<B2>,
) -> Result<Arc<dyn AuthUserProvider<User=AuthUser> + Sync + Send>, AuthBackendError> {
    get_user_provider_u(&vec!(
        backend1.as_ref().map(|b|b.user_provider()),
        backend2.as_ref().map(|b|b.user_provider()),
    ))
}

#[inline(always)]
pub fn usr_prov <
    C,
    B: AuthnBackendAttributes<User=AuthUser,Credentials=C,Error=AuthBackendError>,
> (backend: &Option<B>)
    -> Option<Arc<dyn AuthUserProvider<User=AuthUser> + Sync + Send>> {
    backend.as_ref().map(|b|b.user_provider())
}

#[inline(always)]
pub fn get_user_provider3 <
    C1, C2, C3,
    B1: AuthnBackendAttributes<User=AuthUser,Credentials=C1,Error=AuthBackendError>,
    B2: AuthnBackendAttributes<User=AuthUser,Credentials=C2,Error=AuthBackendError>,
    B3: AuthnBackendAttributes<User=AuthUser,Credentials=C3,Error=AuthBackendError>,
>(
    backend1: &Option<B1>,
    backend2: &Option<B2>,
    backend3: &Option<B3>,
) -> Result<Arc<dyn AuthUserProvider<User=AuthUser> + Sync + Send>, AuthBackendError> {
    get_user_provider_u(&vec!(
        usr_prov(backend1),
        usr_prov(backend2),
        usr_prov(backend3),
    ))
}

/*
#[inline(always)]
pub fn get_user_provider4 <
    B1: AuthnBackendAttributes,
    B2: AuthnBackendAttributes,
    B3: AuthnBackendAttributes,
    B4: AuthnBackendAttributes,
>(
    backend1: &Option<B1>,
    backend2: &Option<B2>,
    backend3: &Option<B3>,
    backend4: &Option<B4>,
) -> Result<Arc<dyn AuthUserProvider<User=AuthUser> + Sync + Send>, AuthBackendError> {
    get_user_provider_u(&vec!(
        usr_prov(backend1),
        usr_prov(backend2),
        usr_prov(backend3),
        usr_prov(backend4),
    ))
}

#[inline(always)]
pub fn get_user_provider5 <
    B1: AuthnBackendAttributes,
    B2: AuthnBackendAttributes,
    B3: AuthnBackendAttributes,
    B4: AuthnBackendAttributes,
    B5: AuthnBackendAttributes,
>(
    backend1: &Option<B1>,
    backend2: &Option<B2>,
    backend3: &Option<B3>,
    backend4: &Option<B4>,
    backend5: &Option<B5>,
) -> Result<Arc<dyn AuthUserProvider<User=AuthUser> + Sync + Send>, AuthBackendError> {
    get_user_provider_u(&vec!(
        usr_prov(backend1),
        usr_prov(backend2),
        usr_prov(backend3),
        usr_prov(backend4),
        usr_prov(backend5),
    ))
}

#[inline(always)]
pub fn get_user_provider6 <
    B1: AuthnBackendAttributes,
    B2: AuthnBackendAttributes,
    B3: AuthnBackendAttributes,
    B4: AuthnBackendAttributes,
    B5: AuthnBackendAttributes,
    B6: AuthnBackendAttributes,
>(
    backend1: &Option<B1>,
    backend2: &Option<B2>,
    backend3: &Option<B3>,
    backend4: &Option<B4>,
    backend5: &Option<B5>,
    backend6: &Option<B6>,
) -> Result<Arc<dyn AuthUserProvider<User=AuthUser> + Sync + Send>, AuthBackendError> {
    get_user_provider_u(&vec!(
        usr_prov(backend1),
        usr_prov(backend2),
        usr_prov(backend3),
        usr_prov(backend4),
        usr_prov(backend5),
        usr_prov(backend6),
    ))
}
*/

pub fn get_user_provider_u(
    possible_user_providers: &Vec<Option<Arc<dyn AuthUserProvider<User=AuthUser> + Sync + Send>>>
) -> Result<Arc<dyn AuthUserProvider<User=AuthUser> + Sync + Send>, AuthBackendError> {

    let all_user_providers: Vec<&Arc<dyn AuthUserProvider<User=AuthUser> + Sync + Send>> =
        possible_user_providers.iter().flat_map(|v|v).collect::<Vec<_>>();

    let users_provider: Arc<dyn AuthUserProvider<User=AuthUser> + Sync + Send> = all_user_providers
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


#[inline(always)]
pub fn unauthenticated_response2 <
    B1: AuthnBackendAttributes<>,
    B2: AuthnBackendAttributes<>,
>(req: &Request,
  backend1: &Option<B1>,
  backend2: &Option<B2>,
) -> Option<Response> {
    backend_propose_auth_action(backend1, req)
        .or_else(|| backend_propose_auth_action(backend2, req))
}

#[inline(always)]
pub fn unauthenticated_response3 <
    B1: AuthnBackendAttributes<>,
    B2: AuthnBackendAttributes<>,
    B3: AuthnBackendAttributes<>,
>(req: &Request,
  backend1: &Option<B1>,
  backend2: &Option<B2>,
  backend3: &Option<B3>,
) -> Option<Response> {
    unauthenticated_response2(req, backend1, backend2)
        .or_else(|| backend_propose_auth_action(backend3, req))
}

#[inline(always)]
pub fn unauthenticated_response4 <
    B1: AuthnBackendAttributes<>,
    B2: AuthnBackendAttributes<>,
    B3: AuthnBackendAttributes<>,
    B4: AuthnBackendAttributes<>,
>(req: &Request,
  backend1: &Option<B1>,
  backend2: &Option<B2>,
  backend3: &Option<B3>,
  backend4: &Option<B4>,
) -> Option<Response> {
    unauthenticated_response3(req, backend1, backend2, backend3)
        .or_else(|| backend_propose_auth_action(backend4, req))
}

#[inline(always)]
pub fn unauthenticated_response5 <
    B1: AuthnBackendAttributes<>,
    B2: AuthnBackendAttributes<>,
    B3: AuthnBackendAttributes<>,
    B4: AuthnBackendAttributes<>,
    B5: AuthnBackendAttributes<>,
>(req: &Request,
  backend1: &Option<B1>,
  backend2: &Option<B2>,
  backend3: &Option<B3>,
  backend4: &Option<B4>,
  backend5: &Option<B5>,
) -> Option<Response> {
    unauthenticated_response4(req, backend1, backend2, backend3, backend4)
        .or_else(|| backend_propose_auth_action(backend5, req))
}

#[inline(always)]
pub fn unauthenticated_response6 <
    B1: AuthnBackendAttributes<>,
    B2: AuthnBackendAttributes<>,
    B3: AuthnBackendAttributes<>,
    B4: AuthnBackendAttributes<>,
    B5: AuthnBackendAttributes<>,
    B6: AuthnBackendAttributes<>,
>(req: &Request,
  backend1: &Option<B1>,
  backend2: &Option<B2>,
  backend3: &Option<B3>,
  backend4: &Option<B4>,
  backend5: &Option<B5>,
  backend6: &Option<B6>,
) -> Option<Response> {
    unauthenticated_response5(req, backend1, backend2, backend3, backend4, backend5)
        .or_else(|| backend_propose_auth_action(backend6, req))
}


pub fn backend_propose_auth_action <
    B: AuthnBackendAttributes,
> (backend: &Option<B>, req: &Request) -> Option<Response> {
    backend.as_ref().and_then(|b|b.propose_authentication_action(req).map(|a|a.into_response()))
}
