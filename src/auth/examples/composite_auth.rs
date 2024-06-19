use std::sync::Arc;

use axum::extract::Request;
use axum::http::StatusCode;
use axum::response::{ IntoResponse, Response };

use axum_login::UserId;
use log::{ error };
use crate::auth::AuthUserProviderError;

use super::auth_user::AuthUserExample;
use super::super::backend::{
    http_basic_auth::HttpBasicAuthBackend,
    login_form_auth::{ LoginFormAuthBackend, LoginFormAuthConfig },
    oauth2_auth::{ OAuth2AuthBackend, OAuth2AuthCredentials },
    psw_auth::PswAuthCredentials,
};
use super::super::{
    backend::{ AuthBackendMode, RequestUserAuthnBackend },
    error::AuthBackendError,
    user_provider::AuthUserProvider,
    psw::PlainPasswordComparator,
    util::composite_util::{ get_user_provider3, unauthenticated_response3 },
    user_provider::InMemAuthUserProvider,
};


#[derive(Clone)]
pub struct CompositeAuthnBackendExample<
    > {
    users_provider: Arc<dyn AuthUserProvider<User=AuthUserExample> + Sync + Send>,
    http_basic_auth_backend: Option<HttpBasicAuthBackend<AuthUserExample,PlainPasswordComparator>>,
    login_form_auth_backend: Option<LoginFormAuthBackend<AuthUserExample,PlainPasswordComparator>>,
    oauth2_backend: Option<OAuth2AuthBackend<AuthUserExample>>,
}

pub fn in_memory_test_users() -> Result<InMemAuthUserProvider<AuthUserExample>, AuthUserProviderError> {
    InMemAuthUserProvider::with_users(vec!(AuthUserExample::new(1, "vovan", "qwerty")))
}

impl CompositeAuthnBackendExample {
    pub fn test_users() -> Result<CompositeAuthnBackendExample, anyhow::Error> {
        let user_provider: Arc<dyn AuthUserProvider<User=AuthUserExample> + Sync + Send> = Arc::new(in_memory_test_users() ?);
        Ok(CompositeAuthnBackendExample {
            http_basic_auth_backend: Some(HttpBasicAuthBackend::new(user_provider.clone(), AuthBackendMode::AuthProposed)),
            login_form_auth_backend: Some(LoginFormAuthBackend::new(user_provider.clone(), LoginFormAuthConfig {
                auth_mode: AuthBackendMode::AuthSupported,
                login_url: "/form/login",
            })),
            users_provider: user_provider,
            oauth2_backend: None,
        })
    }

    pub fn new_raw(
        users_provider: Arc<dyn AuthUserProvider<User=AuthUserExample> + Sync + Send>,
        http_basic_auth_backend: Option<HttpBasicAuthBackend<AuthUserExample,PlainPasswordComparator>>,
        login_form_auth_backend: Option<LoginFormAuthBackend<AuthUserExample,PlainPasswordComparator>>,
        oauth2_backend: Option<OAuth2AuthBackend<AuthUserExample>>,
    ) -> CompositeAuthnBackendExample {
        CompositeAuthnBackendExample { users_provider, http_basic_auth_backend, login_form_auth_backend, oauth2_backend }
    }

    pub fn with_backends(
        http_basic_auth_backend: Option<HttpBasicAuthBackend<AuthUserExample,PlainPasswordComparator>>,
        login_form_auth_backend: Option<LoginFormAuthBackend<AuthUserExample,PlainPasswordComparator>>,
        oauth2_backend: Option<OAuth2AuthBackend<AuthUserExample>>,
    ) -> Result<CompositeAuthnBackendExample, AuthBackendError> {
        let users_provider = get_user_provider3(&http_basic_auth_backend, &login_form_auth_backend, &oauth2_backend) ?;
        Ok(CompositeAuthnBackendExample { users_provider, http_basic_auth_backend, login_form_auth_backend, oauth2_backend })
    }

    // T O D O: Do we need redirection there??
    #[allow(unused_qualifications)]
    pub fn authorize_url(&self) -> Result<(oauth2::url::Url, oauth2::CsrfToken), AuthBackendError> {
        match self.oauth2_backend {
            None => Err(AuthBackendError::NoRequestedBackend),
            Some(ref oauth2_backend) => Ok(oauth2_backend.authorize_url()),
        }
    }

    pub async fn is_authenticated(
        &self,
        auth_session_user: &Option<AuthUserExample>,
        req: Request,
    ) -> (Request, Result<(), Response>) {

        if auth_session_user.is_some() {
            return (req, Ok(()));
        }

        let psw_aut_res_opt: (Request, Result<(), Response>) =
            if let Some(ref backend) = self.http_basic_auth_backend {
                let res: (Request, Result<Option<AuthUserExample>, AuthBackendError>) = backend.call_authenticate_request::<()>(req).await;
                let (req, res) = res;
                let unauthenticated_action_response = map_auth_res_to_is_auth_res(&self, &req, res);
                (req, unauthenticated_action_response)
            } else { (req, Err(StatusCode::UNAUTHORIZED.into_response())) };

        psw_aut_res_opt
    }
}

fn map_auth_res_to_is_auth_res(
    backend: &CompositeAuthnBackendExample,
    req: &Request,
    auth_res: Result<Option<AuthUserExample>, AuthBackendError>,
) -> Result<(), Response> {

    let action_response: Response = unauthenticated_response3(
        req,
        &backend.http_basic_auth_backend,
        &backend.login_form_auth_backend,
        &backend.oauth2_backend
    ).unwrap_or_else(|| StatusCode::UNAUTHORIZED.into_response());

    match auth_res {
        Ok(None) => Err(action_response),
        Ok(_) => Ok(()),
        Err(err) => {
            error!("Authentication error: {:?}", err);
            Err(StatusCode::INTERNAL_SERVER_ERROR.into_response())
        }
    }
}


#[axum::async_trait]
impl axum_login::AuthnBackend for CompositeAuthnBackendExample {
    type User = AuthUserExample;
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
        self.users_provider.get_user_by_principal_identity(user_id).await.map_err(From::from)
    }
}


#[derive(Debug, Clone, serde::Deserialize)]
pub enum CompositeAuthCredentials {
    Password(PswAuthCredentials),
    OAuth(OAuth2AuthCredentials),
}
