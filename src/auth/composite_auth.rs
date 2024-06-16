use std::sync::Arc;

use axum::extract::Request;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use axum_login::UserId;
use log::{ error };
use oauth2::basic::BasicClient;
use psw_auth::PswAuthCredentials;
use crate::auth::auth_backend::{ AuthnBackendAttributes, RequestUserAuthnBackend };
use crate::auth::http_basic_auth::{HttpBasicAuthMode, HttpBasicAuthBackend};
use crate::auth::login_form_auth::{LoginFormAuthBackend, LoginFormAuthMode};

use super::{AuthUserProvider, OAuth2AuthBackend, OAuth2AuthCredentials, psw_auth };
use super::auth_user::AuthUser;
use super::error::AuthBackendError;
use super::psw::PlainPasswordComparator;
use super::mem_user_provider::InMemAuthUserProvider;



// #[derive(Clone)]
pub struct CompositeAuthnBackend <
    > {
    users_provider: Arc<dyn AuthUserProvider<User=AuthUser> + Sync + Send>,
    http_basic_auth_backend: Option<HttpBasicAuthBackend<PlainPasswordComparator>>,
    login_form_auth_backend: Option<LoginFormAuthBackend<PlainPasswordComparator>>,
    oauth2_backend: Option<OAuth2AuthBackend>,
}

impl CompositeAuthnBackend {
    pub fn test_users() -> Result<CompositeAuthnBackend, anyhow::Error> {
        let user_provider: Arc<dyn AuthUserProvider<User=AuthUser> + Sync + Send> = Arc::new(InMemAuthUserProvider::test_users() ?);
        Ok(CompositeAuthnBackend {
            http_basic_auth_backend: Some(HttpBasicAuthBackend::new(user_provider.clone(), HttpBasicAuthMode::BasicAuthProposed)),
            login_form_auth_backend: Some(LoginFormAuthBackend::new(user_provider.clone(), LoginFormAuthMode::LoginFormAuthSupported)),
            users_provider: user_provider,
            oauth2_backend: None,
        })
    }

    pub fn new_raw(
        users_provider: Arc<dyn AuthUserProvider<User=AuthUser> + Sync + Send>,
        http_basic_auth_backend: Option<HttpBasicAuthBackend<PlainPasswordComparator>>,
        login_form_auth_backend: Option<LoginFormAuthBackend<PlainPasswordComparator>>,
        oauth2_backend: Option<OAuth2AuthBackend>,
    ) -> CompositeAuthnBackend {
        CompositeAuthnBackend { users_provider, http_basic_auth_backend, login_form_auth_backend, oauth2_backend }
    }

    pub fn oath2_only(_client: BasicClient) -> CompositeAuthnBackend {
        todo!()
    }

    // TODO: Do we need redirection there??
    #[allow(unused_qualifications)]
    pub fn authorize_url(&self) -> Result<(oauth2::url::Url, oauth2::CsrfToken), AuthBackendError> {
        match self.oauth2_backend {
            None => Err(AuthBackendError::NoRequestedBackend),
            Some(ref oauth2_backend) => Ok(oauth2_backend.authorize_url()),
        }
    }

    pub async fn is_authenticated(
        &self,
        auth_session_user: &Option<AuthUser>,
        req: Request,
    ) -> (Request, Result<(), Response>) {

        use axum::extract::Request;

        if auth_session_user.is_some() {
            return (req, Ok(()));
        }

        let psw_aut_res_opt: (Request, Result<(), Response>) =
            if let Some(ref backend) = self.http_basic_auth_backend {
                let res: (Request, Result<Option<AuthUser>, AuthBackendError>) = backend.call_authenticate_request::<()>(req).await;
                let (req, res) = res;
                let unauthenticated_action_response = map_auth_res_to_is_auth_res(&self, &req, res);
                (req, unauthenticated_action_response)
            } else { (req, Err(StatusCode::UNAUTHORIZED.into_response())) };

        psw_aut_res_opt
    }
}

fn map_auth_res_to_is_auth_res(
    backend: &CompositeAuthnBackend,
    req: &Request,
    auth_res: Result<Option<AuthUser>, AuthBackendError>,
) -> Result<(), Response> {

    let action_response: Response =
        backend_propose_auth_action(&backend.http_basic_auth_backend, req)
        .or_else(|| backend_propose_auth_action(&backend.login_form_auth_backend, req))
        .or_else(|| backend_propose_auth_action(&backend.oauth2_backend, req))
        .unwrap_or_else(|| StatusCode::UNAUTHORIZED.into_response());

    match auth_res {
        Ok(None) => Err(action_response),
        Ok(_) => Ok(()),
        Err(err) => {
            match err {
                // AuthBackendError::NoUser => Err(action),
                // AuthBackendError::NoCredentials => Err(action),
                // AuthBackendError::IncorrectUsernameOrPsw =>  Err(action),
                // AuthBackendError::UserProviderError(_) => {}
                // AuthBackendError::Sqlx(_) => {}
                // AuthBackendError::Reqwest(_) => {}
                // AuthBackendError::OAuth2(_) => {}
                // AuthBackendError::NoRequestedBackend => {}
                // AuthBackendError::TaskJoin(_) => {}
                // AuthBackendError::ConfigError(_) => {}
                other_err => {
                    error!("Authentication error: {:?}", other_err);
                    Err(StatusCode::INTERNAL_SERVER_ERROR.into_response())
                }
            }
        }
    }
}

fn backend_propose_auth_action <
    B: AuthnBackendAttributes,
> (backend: &Option<B>, req: &Request) -> Option<Response> {
    backend.as_ref().and_then(|b|b.propose_authentication_action(req).map(|a|a.into_response()))
}


impl Clone for CompositeAuthnBackend {
    fn clone(&self) -> Self {
        CompositeAuthnBackend {
            users_provider: self.users_provider.clone(),
            http_basic_auth_backend: self.http_basic_auth_backend.clone(),
            login_form_auth_backend: self.login_form_auth_backend.clone(),
            oauth2_backend: self.oauth2_backend.clone(),
        }
    }
    fn clone_from(&mut self, source: &Self) {
        self.http_basic_auth_backend = source.http_basic_auth_backend.clone();
        self.login_form_auth_backend = source.login_form_auth_backend.clone();
        self.oauth2_backend = source.oauth2_backend.clone();
    }
}

#[axum::async_trait]
impl axum_login::AuthnBackend for CompositeAuthnBackend {
    type User = AuthUser;
    type Credentials = CompositeAuthCredentials;
    type Error = AuthBackendError;

    async fn authenticate(&self, creds: Self::Credentials) -> Result<Option<Self::User>, Self::Error> {
        match creds {
            CompositeAuthCredentials::Password(creds) =>
                match self.login_form_auth_backend {
                    None => Err(AuthBackendError::NoRequestedBackend),
                    Some(ref backend) => backend.authenticate(creds).await.map_err(AuthBackendError::from)
                },
            CompositeAuthCredentials::OAuth(creds) =>
                match self.oauth2_backend {
                    None => Err(AuthBackendError::NoRequestedBackend),
                    Some(ref backend) => backend.authenticate(creds).await.map_err(AuthBackendError::from)
                },
        }
    }

    async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        self.users_provider.get_user_by_id(user_id).await.map_err(From::from)
    }
}

// pub type AuthSession = axum_login::AuthSession<CompositeAuthnBackend>;


#[derive(Debug, Clone, serde::Deserialize)]
pub enum CompositeAuthCredentials {
    Password(PswAuthCredentials),
    OAuth(OAuth2AuthCredentials),
}
