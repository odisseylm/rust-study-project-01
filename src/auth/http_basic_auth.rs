use core::fmt;
use std::sync::Arc;

use axum_extra::typed_header::TypedHeaderRejection;
use psw_auth::PswAuthCredentials;
use crate::auth::http::http_unauthenticated_401_response;
use crate::auth::psw_auth::PswAuthBackendImpl;

use super::psw_auth;
use super::auth_backend::{AuthBackendMode, AuthnBackendAttributes, RequestUserAuthnBackend};
use super::auth_user_provider::AuthUserProvider;
use super::auth_user::AuthUser;
use super::psw::PasswordComparator;

use axum_login::AuthnBackend;
use super::axum_login_delegatable::ambassador_impl_AuthnBackend;

#[derive(Clone, ambassador::Delegate)]
#[readonly::make] // should be after 'derive'
#[delegate(axum_login::AuthnBackend, target = "psw_backend")]
pub struct HttpBasicAuthBackend <
    PswComparator: PasswordComparator + Clone + Sync + Send,
> {
    psw_backend: PswAuthBackendImpl<PswComparator>,
    pub auth_mode: AuthBackendMode,
}


impl <
    PswComparator: PasswordComparator + Clone + Sync + Send,
> HttpBasicAuthBackend<PswComparator> {
    pub fn new(
        users_provider: Arc<dyn AuthUserProvider<User = AuthUser> + Sync + Send>,
        auth_mode: AuthBackendMode,
    ) -> HttpBasicAuthBackend<PswComparator> {
        HttpBasicAuthBackend::<PswComparator> {
            psw_backend: PswAuthBackendImpl::new(users_provider.clone()),
            auth_mode,
        }
    }
}


/*
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
    async fn get_user(&self, user_id: &axum_login::UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        self.psw_backend.get_user(user_id).await
    }
}
*/


#[axum::async_trait]
impl <
    PswComparator: PasswordComparator + Clone + Sync + Send,
    > AuthnBackendAttributes for HttpBasicAuthBackend<PswComparator> {
    type ProposeAuthAction = ProposeHttpBasicAuthAction;

    fn user_provider(&self) -> Arc<dyn AuthUserProvider<User=AuthUser> + Sync + Send> {
        self.psw_backend.users_provider()
    }
    fn propose_authentication_action(&self, _: &Request) -> Option<Self::ProposeAuthAction> {
        if let AuthBackendMode::AuthProposed = self.auth_mode
        { Some(ProposeHttpBasicAuthAction) } else { None }
    }
}

pub struct ProposeHttpBasicAuthAction;
impl crate::auth::auth_backend::ProposeAuthAction for ProposeHttpBasicAuthAction { }
#[inherent::inherent]
impl axum::response::IntoResponse for ProposeHttpBasicAuthAction {
    #[allow(dead_code)] // !! It is really used IMPLICITLY !!
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
impl<S> axum::extract::FromRequestParts<S> for BasicAuthCreds where S: Send + Sync {
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

/*
use crate::auth::authn_backend_dyn_wrap::AuthnBackendDynWrapper;

impl <
    PswComparator: PasswordComparator + Clone + Sync + Send,
> TryFrom<HttpBasicAuthBackend<PswComparator>>
for Box<dyn AuthnBackendDynWrapper<Credentials=PswAuthCredentials, Error=AuthBackendError, RealAuthnBackend=HttpBasicAuthBackend<PswComparator>>> {
    type Error = AuthBackendError;

    fn try_from(value: HttpBasicAuthBackend<PswComparator>) -> Result<Self, Self::Error> {
        t o d o!()
    }
}
*/


// TEMP, investigation
#[axum::async_trait]
pub trait RequestUserAuthnBackendDyn : Send + Sync {
    async fn is_req_authenticated(&self, req: Request) -> (Request, Result<Option<AuthUser>, AuthBackendError>);
}

#[axum::async_trait]
impl<
    PswComparator: PasswordComparator + Clone + Sync + Send,
> RequestUserAuthnBackendDyn for HttpBasicAuthBackend<PswComparator> {
    async fn is_req_authenticated(&self, req: Request) -> (Request, Result<Option<AuthUser>, AuthBackendError>) {
        let res = self.call_authenticate_request::<PswComparator>(req).await;
        res
    }
}

use axum::extract::Request;
use super::AuthBackendError;

impl <
    PswComparator: PasswordComparator + Clone + Sync + Send + 'static,
> TryFrom<HttpBasicAuthBackend<PswComparator>>
for Arc<dyn RequestUserAuthnBackendDyn> {
    type Error = AuthBackendError;
    fn try_from(value: HttpBasicAuthBackend<PswComparator>) -> Result<Arc<dyn RequestUserAuthnBackendDyn>, Self::Error> {
        let as_dyn: Arc<dyn RequestUserAuthnBackendDyn> = Arc::new(value);
        Ok(as_dyn)
    }
}

// // Compilation error: error[E0210]: type parameter `A` must be used as the type parameter for some local type (e.g., `MyStruct<A>`)
// impl <
//     A: axum_login::AuthnBackend<User=AuthUser, Credentials=PswAuthCredentials, Error=AuthBackendError>,
// > TryFrom<A>
// for Arc<dyn RequestUserAuthnBackendDyn> {
//     type Error = AuthBackendError;
//     fn try_from(value: &A) -> Result<Arc<dyn RequestUserAuthnBackendDyn>, Self::Error> {
//         Err(AuthBackendError::NoRequestedBackend)
//     }
// }
// impl<T> TryFrom <T>
// for Arc<dyn RequestUserAuthnBackendDyn> {
//     type Error = AuthBackendError;
//     fn try_from(value: T) -> Result<Arc<dyn RequestUserAuthnBackendDyn>, Self::Error> {
//         Err(AuthBackendError::NoRequestedBackend)
//     }
// }

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use super::{ AuthUser, RequestUserAuthnBackendDyn };
    use crate::auth::{ AuthBackendMode, AuthUserProvider, HttpBasicAuthBackend, InMemAuthUserProvider, PlainPasswordComparator };
    use crate::util::TestResultUnwrap;

    #[test]
    fn test_try_into_when_impl() {
        let users = Arc::new(InMemAuthUserProvider::test_users().test_unwrap());
        let users: Arc<dyn AuthUserProvider<User = AuthUser> + Sync + Send> = users;
        let basic_auth = HttpBasicAuthBackend::<PlainPasswordComparator>::new(users, AuthBackendMode::AuthSupported);

        let _as_eee: Result<Arc<dyn RequestUserAuthnBackendDyn>, _> =  basic_auth.try_into();
    }

    /*
    #[test]
    fn test_try_into_when_no_impl() {
        use crate::auth::{ LoginFormAuthBackend, LoginFormAuthConfig };

        let users = Arc::new(InMemAuthUserProvider::test_users().test_unwrap());
        let users: Arc<dyn AuthUserProvider<User = AuthUser> + Sync + Send> = users;
        let basic_auth = LoginFormAuthBackend::<PlainPasswordComparator>::new(
            users, LoginFormAuthConfig { login_url: "/login", auth_mode: AuthBackendMode::AuthSupported });

        let _as_eee: Result<Arc<dyn RequestUserAuthnBackendDyn>, _> =  basic_auth.try_into();
    }
    */

    /*
    use std::sync::Arc;
    // use axum_login::AuthUser;
    use super::AuthUser;
    use crate::auth::{AuthBackendError, AuthBackendMode, BasicAuthCreds, HttpBasicAuthBackend, InMemAuthUserProvider, PlainPasswordComparator, RequestUserAuthnBackend};
    use crate::auth::psw_auth::PswAuthCredentials;

    #[test]
    fn aa() {
        let users = Arc::new(InMemAuthUserProvider::test_users());
        let basic_auth = HttpBasicAuthBackend::<PlainPasswordComparator>::new(users, AuthBackendMode::AuthSupported);

        use RequestUserAuthnBackend;

        // let aa: Option<dyn RequestUserAuthnBackend<PlainPasswordComparator>> = basic_auth.try_into();
        let as_req_user_auth_backend: Result<dyn RequestUserAuthnBackend<User=AuthUser, Credentials=PswAuthCredentials, Error=AuthBackendError, AuthRequestData=BasicAuthCreds>, _> =
            basic_auth.try_into();
        println!("### as_req_user_auth_backend: {:?}", as_req_user_auth_backend)
    }
    */
}