use std::sync::Arc;
use axum_extra::headers::authorization::Basic;
use axum_extra::headers::Authorization as AuthorizationHeader;
use axum_extra::TypedHeader;

use axum_login::UserId;
use oauth2::basic::BasicClient;
use crate::auth::psw_auth::{BasicAuthMode, LoginFormMode};
use crate::rest::auth::AuthUser;

use super::psw_auth;
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
        basic_auth_creds: &Option<TypedHeader<AuthorizationHeader<Basic>>>,
    ) -> bool {

        if let Some(ref psw_backend) = self.psw_backend {
            if psw_backend.is_authenticated(&auth_session_user, &basic_auth_creds).await {
                return true;
            }
        }

        if let Some(ref oauth2_backend) = self.oauth2_backend {
            if oauth2_backend.is_authenticated(&auth_session_user).await {
                return true;
            }
        }

        return false;
    }

}


impl AuthnBackend {

    pub fn oath2_only(_client: BasicClient) -> AuthnBackend {
        todo!()
    }

    pub fn authorize_url(&self) -> Result<(oauth2::url::Url, oauth2::CsrfToken), AuthBackendError> {
        match self.oauth2_backend {
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
