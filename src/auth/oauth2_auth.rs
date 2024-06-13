use std::sync::Arc;
use super::auth_user;
use crate::auth::auth_user::{AuthBackendError, AuthUserProvider, AuthUserProviderError};


#[axum::async_trait]
pub trait Oauth2UserProvider: AuthUserProvider {
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
// #[derive(Debug)]
// #[derive(Clone)]
pub struct AuthBackend {
    // db: SqlitePool,
    user_provider: Arc<dyn Oauth2UserProvider<User = auth_user::AuthUser> + Send + Sync>,
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
        // db: SqlitePool,
        user_provider: Arc<dyn Oauth2UserProvider<User = auth_user::AuthUser> + Send + Sync>,
        client: oauth2::basic::BasicClient,
    ) -> Self {
        Self { user_provider, client }
    }

    pub fn authorize_url(&self) -> (oauth2::url::Url, oauth2::CsrfToken) {
        self.client.authorize_url(oauth2::CsrfToken::new_random).url()
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
        let token_res = self
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

        let user_res = self.user_provider.update_user_access_token(
            user_info.login.as_str(), token_res.access_token().secret().as_str())
            .await
            .map_err(From::<AuthUserProviderError>::from);
        user_res
    }

    async fn get_user(&self, user_id: &axum_login::UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        self.user_provider.get_user_by_id(user_id).await.map_err(From::<AuthUserProviderError>::from)
    }
}
