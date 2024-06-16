use core::fmt;
use std::sync::Arc;

use axum_extra::typed_header::TypedHeaderRejection;
use psw_auth::PswAuthCredentials;
use crate::auth::http::http_unauthenticated_401_response;
use crate::auth::psw_auth::PswAuthBackendImpl;

use super::psw_auth;
use super::auth_backend::{ AuthnBackendAttributes, RequestUserAuthnBackend };
use super::error::AuthBackendError;
use super::auth_user_provider::AuthUserProvider;
use super::auth_user::AuthUser;
use super::psw::PasswordComparator;


#[derive(Copy, Clone, Debug)]
pub enum HttpBasicAuthMode { // TODO: probably we can remove it, or replace with something more abstract ???
    BasicAuthSupported,
    BasicAuthProposed,
}


#[derive(Clone)]
#[readonly::make]
pub struct HttpBasicAuthBackend <
    PswComparator: PasswordComparator + Clone + Sync + Send,
> {
    psw_backend: PswAuthBackendImpl<PswComparator>,
    pub basic_auth_mode: HttpBasicAuthMode,
}


impl <
    PswComparator: PasswordComparator + Clone + Sync + Send,
> HttpBasicAuthBackend<PswComparator> {
    pub fn new(
        users_provider: Arc<dyn AuthUserProvider<User = AuthUser> + Sync + Send>,
        basic_auth_mode: HttpBasicAuthMode,
    ) -> HttpBasicAuthBackend<PswComparator> {
        HttpBasicAuthBackend::<PswComparator> {
            psw_backend: PswAuthBackendImpl::new(users_provider.clone()),
            basic_auth_mode,
        }
    }
}


#[axum::async_trait]
// #[inherent::inherent]
impl<
    PswComparator: PasswordComparator + Clone + Sync + Send,
    > axum_login::AuthnBackend for HttpBasicAuthBackend<PswComparator> {
    type User = AuthUser;
    type Credentials = PswAuthCredentials;
    type Error = AuthBackendError;

    #[inline]
    //noinspection DuplicatedCode
    async fn authenticate(&self, creds: Self::Credentials) -> Result<Option<Self::User>, Self::Error> {
        self.psw_backend.authenticate(creds).await
    }

    #[inline]
    //noinspection DuplicatedCode
    // TODO: try to remove duplicates using Deref, so on.
    async fn get_user(&self, user_id: &axum_login::UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        self.psw_backend.get_user(user_id).await
    }
}


#[axum::async_trait]
// #[inherent::inherent]
impl <
    PswComparator: PasswordComparator + Clone + Sync + Send,
    > AuthnBackendAttributes for HttpBasicAuthBackend<PswComparator> {
    type ProposeAuthAction = ProposeHttpBasicAuthAction;

    fn usr_provider(&self) -> Arc<dyn AuthUserProvider<User=AuthUser> + Sync + Send> {
        self.psw_backend.users_provider()
    }
    fn propose_authentication_action(&self) -> Option<Self::ProposeAuthAction> {
        if let HttpBasicAuthMode::BasicAuthProposed = self.basic_auth_mode
        { Some(ProposeHttpBasicAuthAction) } else { None }
    }
}

pub struct ProposeHttpBasicAuthAction;
impl crate::auth::auth_backend::ProposeAuthAction for ProposeHttpBasicAuthAction { }
#[inherent::inherent]
impl axum::response::IntoResponse for ProposeHttpBasicAuthAction {
    pub fn into_response(self) -> axum::response::Response<axum::body::Body> {
        http_unauthenticated_401_response("Basic")
    }
}


#[derive(Clone)]
pub struct BasicAuthCreds(axum_extra::headers::authorization::Basic);

impl fmt::Debug for BasicAuthCreds {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BasicAuthCreds")
            .field("username", &self.0.username())
            .field("password", &"[...]")
            .finish()
    }
}

#[async_trait::async_trait]
// #[inherent::inherent]
impl<S> axum::extract::FromRequestParts<S> for BasicAuthCreds where S: Send + Sync {
    // type Rejection = core::convert::Infallible; // TODO: put this kind of error to my docs
    type Rejection = TypedHeaderRejection;

    async fn from_request_parts(parts: &mut http::request::Parts, state: &S) -> Result<Self, Self::Rejection> {
        use axum_extra:: { TypedHeader, typed_header::TypedHeaderRejection, headers::{ Authorization, authorization::Basic } };

        let basic_auth: Result<TypedHeader<Authorization<Basic>>, TypedHeaderRejection> =
            TypedHeader::<Authorization<Basic>>::from_request_parts(parts, state).await;

        let basic_auth: Result<BasicAuthCreds, TypedHeaderRejection> = basic_auth.map(|header|BasicAuthCreds(header.0.0.clone()));
        basic_auth
    }
}


#[axum::async_trait]
// #[inherent::inherent]
impl <
    PswComparator: PasswordComparator + Clone + Sync + Send,
> RequestUserAuthnBackend for HttpBasicAuthBackend<PswComparator> {
    type AuthRequestData = BasicAuthCreds;

    async fn authenticate_request<S>(&self, auth_request_data: Self::AuthRequestData) -> Result<Option<Self::User>, Self::Error> {
        use axum_login::AuthnBackend;

        self.authenticate(PswAuthCredentials {
            username: auth_request_data.0.username().to_string(),
            password: auth_request_data.0.password().to_string(),
            next: None,
        }).await
    }
}
