
use std::sync::Arc;

use async_trait::async_trait;

use super::{
    user_provider::AuthUserProvider,
};


#[derive(Debug, Copy, Clone)]
pub enum AuthBackendMode {
    AuthSupported,
    AuthProposed,
}


/// In case of
/// * HTTP Basic authentication it is sending corresponding HTTP header
/// * Login form authentication it is redirecting to HTML form
/// * OAuth also redirecting to form page
///
#[allow(dead_code)] // It is used as type marker (we could directly use IntoResponse, but...)
pub trait ProposeAuthAction : axum::response::IntoResponse {
}


#[async_trait]
pub trait AuthnBackendAttributes : axum_login::AuthnBackend + Clone + Send + Sync {
    type ProposeAuthAction: ProposeAuthAction;
    fn user_provider(&self) -> Arc<dyn AuthUserProvider<User = Self::User> + Sync + Send>;
    fn propose_authentication_action(&self, req: &axum::extract::Request) -> Option<Self::ProposeAuthAction>;
}


#[async_trait]
pub trait RequestUserAuthnBackend : axum_login::AuthnBackend + Clone + Send + Sync {

    // type AuthRequestData: axum::extract::FromRequestParts<_> + Clone + Send + Sync + 'static;
    // fn auth_request_data_extractor<S>() -> Self::AuthRequestDataExtractor<S>;

    type AuthRequestData: Clone + Send + Sync + 'static;

    /// Authenticates the request credentials with the backend if it is present in request.
    /// Since we cannot pass Request (because we cannot clone it) and cannot pass &Request due to
    /// future compilation error, we will extract required request data and pass it to authenticate_request.
    async fn authenticate_request<S>(
        &self,
        auth_request_data: Self::AuthRequestData,
    ) -> Result<Option<Self::User>, Self::Error>;


    // Workaround with returning moved request.
    // Probably not good approach... Hz...
    async fn call_authenticate_request<S>(
        &self,
        req: axum::extract::Request,
    ) -> (axum::extract::Request, Result<Option<Self::User>, Self::Error>)
        where Self::AuthRequestData: axum::extract::FromRequestParts<S> {
        let auth_req_data: Option<&Self::AuthRequestData> = req.extensions().get::<Self::AuthRequestData>();
        let auth_res: Result<Option<Self::User>, Self::Error> =
            if let Some(auth_req_data) = auth_req_data {
                self.authenticate_request::<S>(auth_req_data.clone()).await
            } else { Ok(None) };

        (req, auth_res)
    }

}


pub mod http_basic_auth;
pub mod login_form_auth;
pub mod oauth2_auth;
pub mod psw_auth;
pub mod axum_login_delegatable;
mod authz_backend;
mod std_authz_backend;
mod permission_sets;

pub use http_basic_auth::{ BasicAuthCreds, HttpBasicAuthBackend, ProposeHttpBasicAuthAction };
pub use login_form_auth::{ LoginFormAuthBackend, LoginFormAuthConfig, ProposeLoginFormAuthAction };
pub use oauth2_auth::{ OAuth2AuthBackend, OAuth2AuthCredentials, OAuth2Config, Oauth2ConfigError, OAuth2UserStore };
pub use psw_auth::{ PswAuthCredentials };
