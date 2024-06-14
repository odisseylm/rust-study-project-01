use std::sync::Arc;
use axum::extract::OriginalUri;
use axum_extra::headers::authorization::Basic;
use axum_extra::headers::Authorization as AuthorizationHeader;
use axum_extra::TypedHeader;

use axum_login::UserId;
use oauth2::basic::BasicClient;
use crate::auth::psw_auth::{BasicAuthMode, LoginFormMode};
use crate::rest::auth::AuthUser;

use super::{psw_auth, UnauthenticatedAction};
use super::auth_user;
use super::error::AuthBackendError;
use super::oauth2_auth;
use super::psw::PlainPasswordComparator;
use super::mem_user_provider::InMemAuthUserProvider;


/*
pub async fn is_authenticated (
    auth_session: AuthSession,
    basic_auth_creds: Option<TypedHeader<AuthorizationHeader<Basic>>>,
) -> bool {
    auth_session.backend.is_authenticated(&auth_session.user, &basic_auth_creds)
}
*/


// #[derive(Clone)]
pub struct AuthnBackend <
    > {
    psw_backend: Option<psw_auth::AuthBackend<PlainPasswordComparator>>,
    oauth2_backend: Option<oauth2_auth::AuthBackend>,
}

impl AuthnBackend {
    pub async fn test_users() -> Result<AuthnBackend, anyhow::Error> { // TODO: try to remove async from there
        Ok(AuthnBackend {
            psw_backend: Some(
                psw_auth::AuthBackend::new(
                    Arc::new(InMemAuthUserProvider::new()), BasicAuthMode::BasicAuthProposed, LoginFormMode::LoginFormSupported)),
            oauth2_backend: None,
        })
    }
    pub fn new_raw(
        psw_backend: Option<psw_auth::AuthBackend<PlainPasswordComparator>>,
        oauth2_backend: Option<oauth2_auth::AuthBackend>,
    ) -> AuthnBackend {
        AuthnBackend { psw_backend, oauth2_backend }
    }

    pub async fn is_authenticated (
        &self,
        auth_session_user: &Option<AuthUser>,
        original_uri: &OriginalUri,
        basic_auth_creds: &Option<TypedHeader<AuthorizationHeader<Basic>>>,
    ) -> Result<(), UnauthenticatedAction> {

        let psw_aut_res_opt: Option<Result<(), UnauthenticatedAction>> =
            if let Some(ref psw_backend) = self.psw_backend {
                Some(psw_backend.is_authenticated(&auth_session_user, &original_uri, &basic_auth_creds).await)
            } else { None };

        let oauth2_res_opt: Option<Result<(), UnauthenticatedAction>> = // TODO: do not call if prev is success
            if let Some(ref oauth2_backend) = self.oauth2_backend {
                Some(oauth2_backend.is_authenticated(&auth_session_user, &original_uri).await)
            } else { None };

        // in reverse order to use cheap safe 'pop' method later
        let as_vec = vec!(oauth2_res_opt, psw_aut_res_opt);

        let mut errors: Vec<UnauthenticatedAction> = as_vec.into_iter()
            .filter_map(|r_o|r_o)
            .filter_map(|err|err.err())
            .collect::<Vec<_>>();

        match errors.pop() {
            None => Ok(()),
            Some(err) => Err(err),
        }
    }

}

/*
#[inline]
fn is_any_opt_res_ok2 <
    T1, E1, T2, E2,
    >(r1: &Option<Result<T1,E1>>, r2: &Option<Result<T1,E1>>) -> bool {
    is_opt_res_ok(r1) || is_opt_res_ok(r2)
}
#[inline]
fn is_opt_res_ok<T, E>(r_opt: &Option<Result<T,E>>) -> bool {
    match r_opt {
        None => false,
        Some(ref v) => v.is_ok()
    }
}

#[inline(always)]
fn first_any_opt_res_err2 <T, E> (r1: Option<Result<T,E>>, r2: &Option<Result<T,E>>) -> bool {
    if !is_opt_res_ok(&r1) {
        r1.
    }
}
*/

impl AuthnBackend {

    pub fn oath2_only(_client: BasicClient) -> AuthnBackend {
        todo!()
    }

    pub fn authorize_url(&self) -> Result<(oauth2::url::Url, oauth2::CsrfToken), AuthBackendError> {
        match self.oauth2_backend { // TODO: simplify
            None => Err(AuthBackendError::NoRequestedBackend),
            Some(ref oauth2_backend) => Ok(oauth2_backend.authorize_url()),
        }
    }
}


impl Clone for AuthnBackend {
    fn clone(&self) -> Self {
        AuthnBackend {
            psw_backend: self.psw_backend.clone(),
            oauth2_backend: self.oauth2_backend.clone(),
        }
    }
    fn clone_from(&mut self, source: &Self) {
        self.psw_backend = source.psw_backend.clone();
    }
}

#[axum::async_trait]
impl axum_login::AuthnBackend for AuthnBackend {
    type User = auth_user::AuthUser;
    type Credentials = AuthCredentials;
    type Error = AuthBackendError;

    async fn authenticate(&self, creds: Self::Credentials) -> Result<Option<Self::User>, Self::Error> {
        match creds {
            AuthCredentials::Password(creds) =>
                match self.psw_backend {
                    None => Err(AuthBackendError::NoRequestedBackend),
                    Some(ref backend) => backend.authenticate(creds).await.map_err(AuthBackendError::from)
                },
            AuthCredentials::OAuth(creds) =>
                match self.oauth2_backend {
                    None => Err(AuthBackendError::NoRequestedBackend),
                    Some(ref backend) => backend.authenticate(creds).await.map_err(AuthBackendError::from)
                },
        }
    }

    async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        // expected that app uses only one Users Provider (in all auth backends)
        let res = match self.psw_backend {
            None => Err(AuthBackendError::NoRequestedBackend),
            Some(ref backend) => backend.get_user(user_id).await.map_err(AuthBackendError::from),
        };

        if res.is_ok() { return res }

        // TODO: simplify
        let res = match self.oauth2_backend {
            None => Err(AuthBackendError::NoRequestedBackend),
            Some(ref backend) => backend.get_user(user_id).await.map_err(AuthBackendError::from),
        };

        res
    }
}

pub type AuthSession = axum_login::AuthSession<AuthnBackend>;


#[derive(Debug, Clone, serde::Deserialize)]
pub enum AuthCredentials {
    Password(psw_auth::AuthCredentials),
    OAuth(oauth2_auth::AuthCredentials),
}
