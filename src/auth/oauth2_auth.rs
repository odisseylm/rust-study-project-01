use std::env::VarError;
use std::sync::Arc;
use super::auth_user;
use super::error::AuthBackendError;
use super::auth_user_provider::{ AuthUserProvider, AuthUserProviderError };


#[axum::async_trait]
pub trait Oauth2UserStore: AuthUserProvider {
    async fn update_user_access_token(&self, username: &str, secret_token: &str) -> Result<Option<Self::User>, AuthUserProviderError>;
}


pub type AuthSession = axum_login::AuthSession<AuthBackend>;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct AuthCredentials {
    pub code: String,
    pub old_state: oauth2::CsrfToken,
    pub new_state: oauth2::CsrfToken,
}


#[derive(Debug, Clone)]
pub struct AuthBackend {
    state: Arc<AuthBackendState>,
}


#[derive(Debug)]
struct AuthBackendState {
    user_provider: Arc<dyn Oauth2UserStore<User = auth_user::AuthUser> + Send + Sync>,
    client: oauth2::basic::BasicClient,
}

/*
impl Clone for Backend {
    fn clone(&self) -> Self {
        Backend {
            user_provider: self.user_provider.clone(),
            client: self.client.clone(),
        }
    }
    fn clone_from(&mut self, source: &Self) {
        self.user_provider = source.user_provider.clone();
        self.client = source.client.clone();
    }
}
*/

#[derive(Debug, serde::Deserialize)]
struct UserInfo {
    login: String,
}

impl AuthBackend {
    pub fn new(
        user_provider: Arc<dyn Oauth2UserStore<User = auth_user::AuthUser> + Send + Sync>,
        client: oauth2::basic::BasicClient,
    ) -> Self {
        AuthBackend { state: Arc::new(AuthBackendState {
            user_provider: Arc::clone(&user_provider),
            client: client.clone(),
        }) }
    }

    pub fn authorize_url(&self) -> (oauth2::url::Url, oauth2::CsrfToken) {
        self.state.client.authorize_url(oauth2::CsrfToken::new_random).url()
    }
}

#[axum::async_trait]
impl axum_login::AuthnBackend for AuthBackend {
    type User = auth_user::AuthUser;
    type Credentials = AuthCredentials;
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
        let user_info = reqwest::Client::new()
            .get("https://api.github.com/user")
            // See: https://docs.github.com/en/rest/overview/resources-in-the-rest-api?apiVersion=2022-11-28#user-agent-required
            .header(USER_AGENT.as_str(), "axum-login")
            .header(AUTHORIZATION.as_str(), format!("Bearer {}", token_res.access_token().secret()))
            .send()
            .await
            .map_err(Self::Error::Reqwest)?
            .json::<UserInfo>()
            .await
            .map_err(Self::Error::Reqwest)?;

        let user_res = self.state.user_provider.update_user_access_token(
            user_info.login.as_str(), token_res.access_token().secret().as_str())
            .await
            .map_err(From::<AuthUserProviderError>::from);
        user_res
    }

    async fn get_user(&self, user_id: &axum_login::UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        self.state.user_provider.get_user_by_id(user_id).await.map_err(From::<AuthUserProviderError>::from)
    }
}


pub fn create_basic_client(config: &Oauth2Config) -> Result<oauth2::basic::BasicClient, AuthBackendError> {

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


#[derive(Clone)]
pub struct Oauth2Config {
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

impl Oauth2Config {
    pub fn git_from_env() -> Result<Option<Oauth2Config>, Oauth2ConfigError> {
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

        Ok(Some(Oauth2Config {
            client_id,
            client_secret,
            auth_url,
            token_url,
        }))
    }
}
