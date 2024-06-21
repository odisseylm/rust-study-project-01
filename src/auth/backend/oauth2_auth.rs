use core::fmt::Debug;
use std::env::VarError;
use std::hash::Hash;
use std::sync::Arc;
use axum::extract::OriginalUri;
use oauth2::basic::BasicClient;
use crate::auth::backend::authz_backend::{AuthorizeBackend, PermissionProviderSource};
use crate::auth::permission::{PermissionProvider, PermissionSet};
use crate::auth::permission::empty_perm_provider::{AlwaysAllowedPermSet, EmptyPerm};


use super::super::{
    backend::AuthBackendMode,
    error::AuthBackendError,
    user_provider::{ AuthUserProvider, AuthUserProviderError },
    backend::AuthnBackendAttributes,
};

pub trait OAuth2User {
    fn access_token(&self) -> Option<String>;
    fn access_token_mut(&mut self, access_token: Option<String>);
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
    User: OAuth2User + Debug + Clone + Send + Sync,
    Perm: Hash + Eq + Debug + Clone + Send + Sync = EmptyPerm,
    PermSet: PermissionSet<Permission=Perm> + Debug + Clone + Send + Sync = AlwaysAllowedPermSet<Perm>,
> {
    state: Arc<AuthBackendState<User,Perm,PermSet>>,
}


#[derive(Debug)]
struct AuthBackendState <
    User: OAuth2User + Debug + Clone + Send + Sync,
    Perm: Hash + Eq + Debug + Clone + Send + Sync = EmptyPerm,
    PermSet: PermissionSet<Permission=Perm> + Debug + Clone + Send + Sync = AlwaysAllowedPermSet<Perm>,
> {
    user_provider: Arc<dyn AuthUserProvider<User=User> + Send + Sync>,
    oauth2_user_store: Arc<dyn OAuth2UserStore<User=User> + Send + Sync>,
    config: OAuth2Config,
    client: BasicClient,
    permission_provider: Arc<dyn PermissionProvider<User=User,Permission=Perm,PermissionSet=PermSet>>,
}


#[derive(Debug, serde::Deserialize)]
struct UserInfo {
    login: String,
}

impl <
    Usr: OAuth2User + Debug + Clone + Send + Sync,
    Perm: Hash + Eq + Debug + Clone + Send + Sync,
    PermSet: PermissionSet<Permission=Perm> + Debug + Clone + Send + Sync,
> OAuth2AuthBackend<Usr,Perm,PermSet> {
    pub fn new(
        user_provider: Arc<dyn AuthUserProvider<User=Usr> + Send + Sync>,
        oauth2_user_store: Arc<dyn OAuth2UserStore<User=Usr> + Send + Sync>,
        config: OAuth2Config,
        client: Option<BasicClient>,
        permission_provider: Arc<dyn PermissionProvider<User=Usr,Permission=Perm,PermissionSet=PermSet>>,
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
    Perm: Hash + Eq + Debug + Clone + Send + Sync,
    PermSet: PermissionSet<Permission=Perm> + Debug + Clone + Send + Sync,
> axum_login::AuthnBackend for OAuth2AuthBackend<Usr,Perm,PermSet>
    where Usr: axum_login::AuthUser<Id = String>,
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
            .map_err(Self::Error::OAuth2) ?;

        // Use access token to request user info.
        let config = &self.state.config;
        let user_info = reqwest::Client::new()
            .get(config.get_auth_user_url.as_str())
            // See: https://docs.github.com/en/rest/overview/resources-in-the-rest-api?apiVersion=2022-11-28#user-agent-required
            .header(USER_AGENT.as_str(), "axum-login")
            .header(AUTHORIZATION.as_str(), format!("Bearer {}", token_res.access_token().secret()))
            .send()
            .await
            .map_err(Self::Error::Reqwest)?
            .json::<UserInfo>()
            .await
            .map_err(Self::Error::Reqwest)?;

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

#[derive(Debug, thiserror::Error)]
pub enum Oauth2ConfigError {
    #[error("Oauth2ConfigError({0})")]
    Error(&'static str),
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
            .map_err(|_|Oauth2ConfigError::Error("CLIENT_ID should be provided.")) ?;
        let client_secret = std::env::var("CLIENT_SECRET")
            .map_err(|_|Oauth2ConfigError::Error("CLIENT_SECRET should be provided")) ?;

        let auth_url = "https://github.com/login/oauth/authorize".to_string();
        let token_url = "https://github.com/login/oauth/access_token".to_string();
        let get_auth_user_url = "https://api.github.com/user".to_string();

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
    Perm: Hash + Eq + Debug + Clone + Send + Sync,
    PermSet: PermissionSet<Permission=Perm> + Debug + Clone + Send + Sync,
> AuthnBackendAttributes for OAuth2AuthBackend<Usr,Perm,PermSet>
    where Usr: axum_login::AuthUser<Id = String>,
{
    type ProposeAuthAction = super::login_form_auth::ProposeLoginFormAuthAction;

    fn user_provider(&self) -> Arc<dyn AuthUserProvider<User=Usr> + Sync + Send> {
        self.state.user_provider.clone()
    }
    fn propose_authentication_action(&self, req: &axum::extract::Request) -> Option<Self::ProposeAuthAction> {
        // TODO: add simple oauth2 login form
        let initial_uri: Option<String> = req.extensions().get::<OriginalUri>().map(|uri|uri.to_string());
        Some(super::login_form_auth::ProposeLoginFormAuthAction { login_url: Some(self.state.config.login_url), initial_url: initial_uri })
    }
}

#[axum::async_trait]
impl<
    Usr: OAuth2User + Debug + Clone + Send + Sync,
    Perm: Hash + Eq + Debug + Clone + Send + Sync,
    PermSet: PermissionSet<Permission=Perm> + Debug + Clone + Send + Sync
> PermissionProviderSource for OAuth2AuthBackend<Usr,Perm,PermSet>
    where Usr: axum_login::AuthUser<Id = String> {
    type User = Usr;
    type Permission = Perm;
    type PermissionSet = PermSet;

    #[inline]
    //noinspection DuplicatedCode
    fn permission_provider(&self) -> Arc<dyn PermissionProvider<User=Self::User, Permission=Self::Permission, PermissionSet=Self::PermissionSet>> {
        self.state.permission_provider.clone()
    }
}
#[axum::async_trait]
impl<
    Usr: OAuth2User + Debug + Clone + Send + Sync,
    Perm: Hash + Eq + Debug + Clone + Send + Sync,
    PermSet: PermissionSet<Permission=Perm> + Debug + Clone + Send + Sync
> AuthorizeBackend for OAuth2AuthBackend<Usr,Perm,PermSet>
    where Usr: axum_login::AuthUser<Id = String> {
    //noinspection DuplicatedCode
}
