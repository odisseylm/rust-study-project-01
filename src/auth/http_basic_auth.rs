use core::fmt;
use std::sync::Arc;

// use axum_login;
// use axum_login::{AuthnBackend, tower_sessions};
// use axum_extra::TypedHeader;
// use tower::ServiceExt;
// use axum_extra::headers::authorization::Basic;
use axum_extra::typed_header::TypedHeaderRejection;
use psw_auth::PswAuthCredentials;
use crate::auth::psw_auth::PswAuthBackendImpl;
// use crate::auth::UnauthenticatedAction;

use super::psw_auth;
use super::auth_backend::RequestUserAuthnBackend;
use super::error::AuthBackendError;
use super::auth_user_provider::AuthUserProvider;
use super::auth_user::AuthUser;
use super::psw::PasswordComparator;


// pub type HttpBasicAuthSession <
//     PswComparator, // : PasswordComparator + Clone + Sync + Send,
// > = axum_login::AuthSession<HttpBasicAuthBackend<PswComparator>>;


#[derive(Copy, Clone, Debug)]
pub enum HttpBasicAuthMode {
    // BasicAuthIgnored,
    BasicAuthSupported,
    BasicAuthProposed,
}
// impl HttpBasicAuthMode {
//     pub fn ignored(&self)->bool {
//         if let HttpBasicAuthMode::BasicAuthIgnored = self { true }
//         else { false }
//     }
// }


#[derive(Clone)]
#[readonly::make]
pub struct HttpBasicAuthBackend <
    PswComparator: PasswordComparator + Clone + Sync + Send,
> {
    psw_backend: PswAuthBackendImpl<PswComparator>,
    pub basic_auth_mode: HttpBasicAuthMode,
    // _pd: PhantomData<PswComparator>,
}


/*
impl<
    PswComparator: PasswordComparator + Clone + Sync + Send,
> Clone for HttpBasicAuthBackend<PswComparator> {
    fn clone(&self) -> Self {
        AuthBackend::<PswComparator> {
            users_provider: self.users_provider.clone(),
            basic_auth_mode: self.basic_auth_mode,
            _pd: PhantomData,
        }
    }
    fn clone_from(&mut self, source: &Self) {
        self.users_provider = source.users_provider.clone();
    }
}
*/


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
impl<
    PswComparator: PasswordComparator + Clone + Sync + Send,
    > axum_login::AuthnBackend for HttpBasicAuthBackend<PswComparator> {
    type User = AuthUser;
    type Credentials = PswAuthCredentials;
    type Error = AuthBackendError;

    #[inline]
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
impl<S> axum::extract::FromRequestParts<S> for BasicAuthCreds where S: Send + Sync {
    // type Rejection = core::convert::Infallible; // TODO: put this kind of error to my docs
    type Rejection = TypedHeaderRejection;

    async fn from_request_parts(parts: &mut http::request::Parts, state: &S) -> Result<Self, Self::Rejection> {
        // use axum::extract::OriginalUri;
        //let original_uri: Option<OriginalUri> = OriginalUri::from_request_parts(parts, state).await.ok();

        use axum_extra:: { TypedHeader, typed_header::TypedHeaderRejection, headers::{ Authorization, authorization::Basic } };

        let basic_auth: Result<TypedHeader<Authorization<Basic>>, TypedHeaderRejection> =
            TypedHeader::<Authorization<Basic>>::from_request_parts(parts, state).await;
        // let basic_auth: Option<Basic> =
        //     if let Ok(TypedHeader(Authorization(basic_auth))) = basic_auth { Some(basic_auth) }
        //     else { None };

        let basic_auth: Result<BasicAuthCreds, TypedHeaderRejection> = basic_auth.map(|header|BasicAuthCreds(header.0.0.clone()));
        basic_auth
    }
}


#[axum::async_trait]
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
