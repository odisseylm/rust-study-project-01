use core::fmt::Debug;
use core::hash::Hash;
use std::env::VarError;
use std::sync::Arc;

use oauth2::basic::BasicClient;
use mvv_common::backtrace::{backtrace, BacktraceCell};
use crate::{
    backend::{
        RequestAuthenticated, AuthBackendMode, AuthnBackendAttributes,
        authz_backend::{ AuthorizeBackend, PermissionProviderSource, },
    },
    http::req_original_uri,
    permission::{
        PermissionProvider, PermissionSet,
        empty_perm_provider::{ AlwaysAllowedPermSet, EmptyPerm, },
    },
    user_provider::{ AuthUserProvider, AuthUserProviderError, },
    error::AuthBackendError,
    SecureString,
};
//--------------------------------------------------------------------------------------------------



pub trait OAuth2User {
    fn access_token(&self) -> Option<SecureString>;
    fn access_token_mut(&mut self, access_token: Option<SecureString>);
}

#[axum::async_trait]
pub trait OAuth2UserStore: AuthUserProvider {
    async fn update_user_access_token(&self, user_principal_id: <<Self as AuthUserProvider>::User as axum_login::AuthUser>::Id, secret_token: &str) -> Result<Option<Self::User>, AuthUserProviderError>;
}


#[derive(Debug, Clone, serde::Deserialize)]
pub struct OAuth2AuthCredentials {
    pub code: String,
    pub old_state: oauth2::CsrfToken,
    pub new_state: oauth2::CsrfToken,
}


#[derive(Debug, Clone)]
#[readonly::make]
pub struct OAuth2AuthBackend <
    User: Debug + Clone,
    PermSet: PermissionSet + Clone = AlwaysAllowedPermSet<EmptyPerm>,
>   where
        <PermSet as PermissionSet>::Permission : Hash + Eq,
{
    state: Arc<AuthBackendState<User,PermSet>>,
}


#[derive(Debug)]
struct AuthBackendState <
    User: Debug,
    PermSet: PermissionSet = AlwaysAllowedPermSet<EmptyPerm>,
> {
    user_provider: Arc<dyn AuthUserProvider<User=User> + Send + Sync>,
    oauth2_user_store: Arc<dyn OAuth2UserStore<User=User> + Send + Sync>,
    config: OAuth2Config,
    client: BasicClient,
    permission_provider: Arc<dyn PermissionProvider<User=User,Permission=<PermSet as PermissionSet>::Permission,PermissionSet=PermSet> + Send + Sync>,
}


#[derive(Debug, serde::Deserialize)]
struct UserInfo {
    login: String,
}

impl <
    Usr: OAuth2User + Debug + Clone,
    PermSet: PermissionSet + Clone,
> OAuth2AuthBackend<Usr,PermSet>
    where
        <PermSet as PermissionSet>::Permission : Hash + Eq,
{
    pub fn new(
        user_provider: Arc<dyn AuthUserProvider<User=Usr> + Send + Sync>,
        oauth2_user_store: Arc<dyn OAuth2UserStore<User=Usr> + Send + Sync>,
        config: OAuth2Config,
        client: Option<BasicClient>,
        permission_provider: Arc<dyn PermissionProvider<User=Usr,Permission=<PermSet as PermissionSet>::Permission,PermissionSet=PermSet> + Send + Sync>,
    ) -> Result<Self, AuthBackendError> {

        // ?? Strange... no Option.or_else_res() function.
        // let client: Result<BasicClient,AuthBackendError> = client.clone()
        //     .ok_or_else(||AuthBackendError::NoRequestedBackend)
        //     .or_else(|_| create_basic_client(&config));

        let client: BasicClient = match client {
            None => create_basic_client(&config) ?,
            Some(client) => client,
        };
        Ok(OAuth2AuthBackend {
            state: Arc::new(AuthBackendState {
                user_provider: Arc::clone(&user_provider),
                oauth2_user_store: Arc::clone(&oauth2_user_store),
                config,
                client,
                permission_provider,
            })
        })
    }

    #[allow(unused_qualifications)]
    pub fn authorize_url(&self) -> (oauth2::url::Url, oauth2::CsrfToken) {
        self.state.client.authorize_url(oauth2::CsrfToken::new_random).url()
    }
}

#[axum::async_trait]
impl <
    Usr: OAuth2User + Debug + Clone + Send + Sync,
    PermSet: PermissionSet + Clone,
> axum_login::AuthnBackend for OAuth2AuthBackend<Usr,PermSet>
    where
        <PermSet as PermissionSet>::Permission : Hash + Eq,
        Usr: axum_login::AuthUser<Id = String>,
{
    type User = Usr;
    type Credentials = OAuth2AuthCredentials;
    type Error = AuthBackendError;

    async fn authenticate(&self, creds: Self::Credentials) -> Result<Option<Self::User>, Self::Error> {

        use axum::http::header::{ AUTHORIZATION, USER_AGENT };
        use oauth2::{ reqwest::async_http_client, TokenResponse };


        // Ensure the CSRF state has not been tampered with.
        if creds.old_state.secret() != creds.new_state.secret() {
            return Ok(None);
        };

        // Process authorization code, expecting a token response back.
        let token_res = self.state
            .client
            .exchange_code(oauth2::AuthorizationCode::new(creds.code))
            .request_async(async_http_client)
            .await
            .map_err(|err|Self::Error::OAuth2(err, backtrace())) ?;

        // Use access token to request user info.
        let config = &self.state.config;
        let user_info = reqwest::Client::new()
            .get(config.get_auth_user_url.as_str())
            // See: https://docs.github.com/en/rest/overview/resources-in-the-rest-api?apiVersion=2022-11-28#user-agent-required
            .header(USER_AGENT.as_str(), "axum-login")
            .header(AUTHORIZATION.as_str(), format!("Bearer {}", token_res.access_token().secret()))
            .send()
            .await
            .map_err(|err|Self::Error::Reqwest(err, backtrace())) ?
            .json::<UserInfo>()
            .await
            .map_err(|err|Self::Error::Reqwest(err, backtrace())) ?;

        let user_res = self.state.oauth2_user_store.update_user_access_token(
            user_info.login.clone(), token_res.access_token().secret().as_str())
            .await
            .map_err(From::<AuthUserProviderError>::from);
        user_res
    }

    async fn get_user(&self, user_id: &axum_login::UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        self.state.user_provider.get_user_by_principal_identity(user_id).await.map_err(From::<AuthUserProviderError>::from)
    }
}


#[axum::async_trait]
impl <
    Usr: OAuth2User + Debug + Clone + Send + Sync,
    PermSet: PermissionSet + Clone,
> RequestAuthenticated for OAuth2AuthBackend<Usr,PermSet>
    where
        <PermSet as PermissionSet>::Permission : Hash + Eq,
        Usr: axum_login::AuthUser<Id = String>,
{ }


pub fn create_basic_client(config: &OAuth2Config) -> Result<BasicClient, AuthBackendError> {

    let orig_config = config;
    let config = config.clone();

    use oauth2::{AuthUrl, ClientId, ClientSecret, TokenUrl};
    use oauth2::basic::BasicClient;

    let client_id = ClientId::new(config.client_id);
    let client_secret = ClientSecret::new(config.client_secret);
    let auth_url = AuthUrl::new(config.auth_url)
        .map_err(|_|AuthBackendError::ConfigError(anyhow::anyhow!("Incorrect auth_url [{}]", orig_config.auth_url))) ?;
    let token_url = TokenUrl::new(config.token_url)
        .map_err(|_|AuthBackendError::ConfigError(anyhow::anyhow!("Incorrect token_url [{}]", orig_config.token_url))) ?;

    Ok(BasicClient::new(client_id, Some(client_secret), auth_url, Some(token_url)))
}


#[derive(Debug, Clone)]
pub struct OAuth2Config {
    pub auth_mode: AuthBackendMode,
    pub login_url: &'static str,

    // Get the authenticated user. See https://docs.github.com/en/rest/users/users?apiVersion=2022-11-28#get-the-authenticated-user
    pub get_auth_user_url: String,
    pub client_id: String,
    pub client_secret: String,
    pub auth_url: String,
    pub token_url: String,
}

#[derive(
    Debug,
    thiserror::Error,
    mvv_error_macro::ThisErrorFromWithBacktrace,
    mvv_error_macro::ThisErrorBacktraceSource,
)]
pub enum Oauth2ConfigError {
    #[error("Oauth2ConfigError({0})")]
    Error(&'static str, BacktraceCell),
}

impl OAuth2Config {
    pub fn git_from_env() -> Result<Option<OAuth2Config>, Oauth2ConfigError> {
        let client_id_env = std::env::var("CLIENT_ID");
        let client_secret_env = std::env::var("CLIENT_SECRET");

        let no_client_id_env = client_id_env.err() == Some(VarError::NotPresent);
        let no_client_secret_env = client_secret_env.err() == Some(VarError::NotPresent);
        if no_client_id_env && no_client_secret_env {
            return Ok(None);
        }

        let client_id = std::env::var("CLIENT_ID")
            .map_err(|_|Oauth2ConfigError::Error(
                "CLIENT_ID should be provided.", backtrace())) ?;
        let client_secret = std::env::var("CLIENT_SECRET")
            .map_err(|_|Oauth2ConfigError::Error(
                "CLIENT_SECRET should be provided", backtrace())) ?;

        let auth_url = "https://github.com/login/oauth/authorize".to_owned();
        let token_url = "https://github.com/login/oauth/access_token".to_owned();
        let get_auth_user_url = "https://api.github.com/user".to_owned();

        Ok(Some(OAuth2Config {
            auth_mode: AuthBackendMode::AuthSupported,
            login_url: "/oauth2/login",
            get_auth_user_url,
            client_id,
            client_secret,
            auth_url,
            token_url,
        }))
    }
}


#[axum::async_trait]
impl <
    Usr: OAuth2User + Debug + Clone + Send + Sync,
    PermSet: PermissionSet + Clone,
> AuthnBackendAttributes for OAuth2AuthBackend<Usr,PermSet>
    where
        <PermSet as PermissionSet>::Permission : Hash + Eq,
        Usr: axum_login::AuthUser<Id = String>,
{
    type ProposeAuthAction = super::login_form_auth::ProposeLoginFormAuthAction;

    fn user_provider(&self) -> Arc<dyn AuthUserProvider<User=Usr> + Sync + Send> {
        Arc::clone(&self.state.user_provider)
    }
    fn user_provider_ref<'a>(&'a self) -> &'a Arc<dyn AuthUserProvider<User=Self::User> + Sync + Send> {
        &self.state.user_provider
    }

    fn propose_authentication_action(&self, req: &axum::extract::Request) -> Option<Self::ProposeAuthAction> {
        // TODO: add simple oauth2 login form
        let initial_uri: Option<String> = req_original_uri(&req);
        Some(super::login_form_auth::ProposeLoginFormAuthAction { login_url: Some(self.state.config.login_url), initial_url: initial_uri })
    }
}

#[axum::async_trait]
impl<
    Usr: OAuth2User + Debug + Clone + Send + Sync,
    PermSet: PermissionSet + Clone,
> PermissionProviderSource for OAuth2AuthBackend<Usr,PermSet>
    where
        <PermSet as PermissionSet>::Permission : Hash + Eq,
        Usr: axum_login::AuthUser<Id = String>,
{
    type User = Usr;
    type Permission = <PermSet as PermissionSet>::Permission;
    type PermissionSet = PermSet;

    #[inline]
    //noinspection DuplicatedCode
    fn permission_provider(&self) -> Arc<dyn PermissionProvider<User=Self::User, Permission=Self::Permission, PermissionSet=Self::PermissionSet> + Send + Sync> {
        Arc::clone(&self.state.permission_provider)
    }
    #[inline] // for local/non-async usage
    //noinspection DuplicatedCode
    fn permission_provider_ref<'a>(&'a self) -> &'a Arc<dyn PermissionProvider<User=Self::User, Permission=Self::Permission, PermissionSet=Self::PermissionSet> + Send + Sync> {
        &self.state.permission_provider
    }
}
#[axum::async_trait]
impl<
    Usr: OAuth2User + Debug + Clone + Send + Sync,
    PermSet: PermissionSet + Clone,
> AuthorizeBackend for OAuth2AuthBackend<Usr,PermSet>
    where
        <PermSet as PermissionSet>::Permission : Hash + Eq,
        Usr: axum_login::AuthUser<Id = String>,
{ }
