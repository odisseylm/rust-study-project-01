use std::sync::Arc;
use async_trait::async_trait;
use crate::auth::{AuthUser, AuthUserProvider};


/// In case of
/// * HTTP Basic authentication it is sending corresponding HTTP header
/// * Login form authentication it is redirecting to HTML form
/// * OAuth also redirecting to form page
///
pub trait ProposeAuthAction : axum::response::IntoResponse {
}


#[async_trait]
pub trait AuthnBackendAttributes : axum_login::AuthnBackend + Clone + Send + Sync {
    type ProposeAuthAction: ProposeAuthAction;
    fn usr_provider(&self) -> Arc<dyn AuthUserProvider<User = AuthUser> + Sync + Send>;
    fn propose_authentication_action(&self) -> Option<Self::ProposeAuthAction>;
}


#[async_trait]
pub trait RequestUserAuthnBackend : axum_login::AuthnBackend + Clone + Send + Sync {

    // type AuthRequestData: axum::extract::FromRequestParts<_> + Clone + Send + Sync + 'static;
    type AuthRequestData: Clone + Send + Sync + 'static;
    //type AuthRequestDataExtractor<S>: axum::extract::FromRequestParts<S>; // where S + Sync + Send;
    // fn auth_request_data_extractor<S>() -> Self::AuthRequestDataExtractor<S>;
    // fn auth_request_data_extractor22<S>() -> Self::AuthRequestData where Self::AuthRequestData: axum::extract::FromRequestParts<S>;

    /// Authenticates the request credentials with the backend if it is present in request.
    /// Since we cannot pass Request (because we cannot clone it) and cannot pass &Request due to
    /// future compilation error, we will extract required request data and pass it to authenticate_request.
    async fn authenticate_request<S>(
        &self,
        // creds: Self::Credentials,
        auth_request_data: Self::AuthRequestData,
    ) -> Result<Option<Self::User>, Self::Error>;

    // fn auth_request_data_extractor<S>() -> Self::AuthRequestDataExtractor<S>;

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
                // TODO: try to avoid 'clone'
                self.authenticate_request::<S>(auth_req_data.clone()).await
            } else { Ok(None) };

        (req, auth_res)
    }

}


