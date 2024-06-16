use async_trait::async_trait;


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
        //use axum::extract::FromRequestParts;
        let auth_req_data: Option<&Self::AuthRequestData> = req.extensions().get::<Self::AuthRequestData>();
        let auth_res: Result<Option<Self::User>, Self::Error> =
            if let Some(auth_req_data) = auth_req_data {
                // TODO: try to avoid 'clone'
                self.authenticate_request::<S>(auth_req_data.clone()).await
            // } else { Err(AuthBackendError::NoCredentials) };
            } else { Ok(None) };

        (req, auth_res)
    }

}


