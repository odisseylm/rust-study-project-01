use std::sync::Arc;

use axum::extract::Request;
use axum::http::StatusCode;
use axum::response::{ IntoResponse, Response };

use axum_login::UserId;
use log::{ error };
use crate::auth::AuthUserProviderError;
use crate::auth::backend::authz_backend::{AuthorizeBackend, PermissionProviderSource};
use crate::auth::permission::PermissionProvider;
use crate::auth::util::composite_util::get_permission_provider3;

use super::auth_user::{AuthUserExample, AuthUserExamplePswExtractor};
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

pub type Role = crate::auth::permission::predefined::Role;
pub type RolePermissionsSet = crate::auth::permission::predefined::RolePermissionsSet;


#[derive(Clone)]
pub struct CompositeAuthnBackendExample<
    > {
    users_provider: Arc<dyn AuthUserProvider<User=AuthUserExample> + Sync + Send>,
    permission_provider: Arc<dyn PermissionProvider<User=AuthUserExample,Permission=Role,PermissionSet=RolePermissionsSet> + Sync + Send>,
    http_basic_auth_backend: Option<HttpBasicAuthBackend<AuthUserExample,PlainPasswordComparator,Role,RolePermissionsSet>>,
    login_form_auth_backend: Option<LoginFormAuthBackend<AuthUserExample,PlainPasswordComparator,Role,RolePermissionsSet>>,
    oauth2_backend: Option<OAuth2AuthBackend<AuthUserExample,Role,RolePermissionsSet>>,
}

pub fn in_memory_test_users() -> Result<InMemAuthUserProvider<AuthUserExample,Role,RolePermissionsSet,AuthUserExamplePswExtractor>, AuthUserProviderError> {
    InMemAuthUserProvider::with_users(vec!(AuthUserExample::new(1, "vovan", "qwerty")))
}

impl CompositeAuthnBackendExample {
    pub fn test_users() -> Result<CompositeAuthnBackendExample, anyhow::Error> {
        let in_mem_users = Arc::new(in_memory_test_users() ?);
        let user_provider: Arc<dyn AuthUserProvider<User=AuthUserExample> + Sync + Send> = in_mem_users.clone();
        let permission_provider: Arc<dyn PermissionProvider<User=AuthUserExample,Permission=Role,PermissionSet=RolePermissionsSet> + Sync + Send> = in_mem_users.clone();
        Ok(CompositeAuthnBackendExample {
            http_basic_auth_backend: Some(HttpBasicAuthBackend::new(user_provider.clone(), AuthBackendMode::AuthProposed, permission_provider.clone())),
            login_form_auth_backend: Some(LoginFormAuthBackend::new(user_provider.clone(), LoginFormAuthConfig {
                auth_mode: AuthBackendMode::AuthSupported,
                login_url: "/form/login",
            }, permission_provider.clone())),
            users_provider: user_provider,
            permission_provider,
            oauth2_backend: None,
        })
    }

    pub fn new_raw(
        users_provider: Arc<dyn AuthUserProvider<User=AuthUserExample> + Sync + Send>,
        permission_provider: Arc<dyn PermissionProvider<User=AuthUserExample,Permission=Role,PermissionSet=RolePermissionsSet> + Sync + Send>,
        http_basic_auth_backend: Option<HttpBasicAuthBackend<AuthUserExample,PlainPasswordComparator,Role,RolePermissionsSet>>,
        login_form_auth_backend: Option<LoginFormAuthBackend<AuthUserExample,PlainPasswordComparator,Role,RolePermissionsSet>>,
        oauth2_backend: Option<OAuth2AuthBackend<AuthUserExample,Role,RolePermissionsSet>>,
    ) -> CompositeAuthnBackendExample {
        CompositeAuthnBackendExample { users_provider, http_basic_auth_backend, login_form_auth_backend, oauth2_backend, permission_provider }
    }

    pub fn with_backends(
        http_basic_auth_backend: Option<HttpBasicAuthBackend<AuthUserExample,PlainPasswordComparator,Role,RolePermissionsSet>>,
        login_form_auth_backend: Option<LoginFormAuthBackend<AuthUserExample,PlainPasswordComparator,Role,RolePermissionsSet>>,
        oauth2_backend: Option<OAuth2AuthBackend<AuthUserExample,Role,RolePermissionsSet>>,
    ) -> Result<CompositeAuthnBackendExample, AuthBackendError> {
        let users_provider = get_user_provider3(&http_basic_auth_backend, &login_form_auth_backend, &oauth2_backend) ?;
        let permission_provider = get_permission_provider3(&http_basic_auth_backend, &login_form_auth_backend, &oauth2_backend) ?;
        Ok(CompositeAuthnBackendExample { users_provider, http_basic_auth_backend, login_form_auth_backend, oauth2_backend, permission_provider })
    }

    // T O D O: Do we need redirection there??
    #[allow(unused_qualifications)]
    pub fn authorize_url(&self) -> Result<(oauth2::url::Url, oauth2::CsrfToken), AuthBackendError> {
        match self.oauth2_backend {
            None => Err(AuthBackendError::NoRequestedBackend),
            Some(ref oauth2_backend) => Ok(oauth2_backend.authorize_url()),
        }
    }

    /*
    pub async fn do_is_authenticated2(
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
    */

    pub async fn do_authenticate(
        &self,
        req: Request,
        auth_session: axum_login::AuthSession<CompositeAuthnBackendExample>,
    ) -> (Request, axum_login::AuthSession<CompositeAuthnBackendExample>, Result<Option<AuthUserExample>,Response>) {

        if auth_session.user.is_some() {
            let user = auth_session.user.clone();
            return (req, auth_session, Ok(user));
        }

        let psw_aut_res_opt: (Request, axum_login::AuthSession<CompositeAuthnBackendExample>, Result<Option<AuthUserExample>, Response>) =
            if let Some(ref backend) = self.http_basic_auth_backend {
                let res: (Request, Result<Option<AuthUserExample>, AuthBackendError>) = backend.do_authenticate_request::<()>(req).await;
                let (req, res) = res;
                let unauthenticated_action_response = map_auth_res_to_is_auth_res(&self, &req, res);
                (req, auth_session, unauthenticated_action_response)
            } else { (req, auth_session, Err(StatusCode::UNAUTHORIZED.into_response())) };

        psw_aut_res_opt
    }

    /*
    pub async fn is_authorized(
        &self,
        auth_session_user: &Option<AuthUserExample>,
        req: Request,
        role: Role,
    ) -> (Request, Result<(), Response>) {
    }
    */
}

fn map_auth_res_to_is_auth_res(
    backend: &CompositeAuthnBackendExample,
    req: &Request,
    auth_res: Result<Option<AuthUserExample>, AuthBackendError>,
) -> Result<Option<AuthUserExample>, Response> {

    // It can be
    // * redirection (in case of login form auth)
    // * request authentication by browser (in case of HTTP Basic auth)
    // * so on
    let unauthenticated_action_response: Response = unauthenticated_response3(
        req,
        &backend.http_basic_auth_backend,
        &backend.login_form_auth_backend,
        &backend.oauth2_backend
    ).unwrap_or_else(|| StatusCode::UNAUTHORIZED.into_response());

    match auth_res {
        Ok(None) => Err(unauthenticated_action_response),
        Ok(user_opt) => Ok(user_opt),
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



#[axum::async_trait]
impl PermissionProviderSource for CompositeAuthnBackendExample {
    type User = AuthUserExample;
    type Permission = Role;
    type PermissionSet = RolePermissionsSet;

    #[inline]
    //noinspection DuplicatedCode
    fn permission_provider(&self) -> Arc<dyn PermissionProvider<User=AuthUserExample,Permission=Role,PermissionSet=RolePermissionsSet>> {
        self.permission_provider.clone()
    }
}
#[axum::async_trait]
impl AuthorizeBackend for CompositeAuthnBackendExample { }
