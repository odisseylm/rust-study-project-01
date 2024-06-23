use core::fmt::Debug;
use std::hash::Hash;
use std::sync::Arc;


use super::psw_auth::{ PswAuthBackendImpl, PswAuthCredentials, PswUser };
use super::super::{
    util::http::http_unauthenticated_401_response,
    backend::{ AuthBackendMode, AuthnBackendAttributes, ProposeAuthAction },
    user_provider::AuthUserProvider,
    psw::PasswordComparator,
};

use axum_login::AuthnBackend;
// use super::axum_login_delegatable::ambassador_impl_AuthnBackend;

#[derive(Clone)]
// #[derive(Clone, ambassador::Delegate)]
#[readonly::make] // should be after 'derive'
// #[delegate(axum_login::AuthnBackend, target = "psw_backend")]
pub struct HttpBasicAuthBackend <
    User: axum_login::AuthUser + PswUser,
    PswComparator: PasswordComparator + Debug + Clone + Send + Sync,
    Perm: Hash + Eq + Debug + Clone + Send + Sync = EmptyPerm,
    PermSet: PermissionSet<Permission=Perm> + Debug + Clone + Send + Sync = AlwaysAllowedPermSet<Perm>,
> {
    psw_backend: PswAuthBackendImpl<User,PswComparator,Perm,PermSet>,
    pub auth_mode: AuthBackendMode,
}


impl <
    Usr: axum_login::AuthUser + PswUser,
    PswComp: PasswordComparator + Debug + Clone + Send + Sync,
    Perm: Hash + Eq + Debug + Clone + Send + Sync,
    PermSet: PermissionSet<Permission=Perm> + Debug + Clone + Send + Sync,
> HttpBasicAuthBackend<Usr,PswComp,Perm,PermSet>
    where Usr: axum_login::AuthUser<Id = String>,
{
    pub fn new(
        users_provider: Arc<dyn AuthUserProvider<User=Usr> + Send + Sync>,
        auth_mode: AuthBackendMode,
        permission_provider: Arc<dyn PermissionProvider<User=Usr,Permission=Perm,PermissionSet=PermSet> + Send + Sync>,
    ) -> HttpBasicAuthBackend<Usr,PswComp,Perm,PermSet> {
        HttpBasicAuthBackend::<Usr,PswComp,Perm,PermSet> {
            psw_backend: PswAuthBackendImpl::new(
                users_provider,
                permission_provider,
            ),
            auth_mode,
        }
    }
}


#[axum::async_trait]
impl <
    Usr: axum_login::AuthUser + PswUser,
    PswComp: PasswordComparator + Debug + Clone + Send + Sync,
    Perm: Hash + Eq + Debug + Clone + Send + Sync,
    PermSet: PermissionSet<Permission=Perm> + Debug + Clone + Send + Sync,
> AuthnBackend for HttpBasicAuthBackend<Usr,PswComp,Perm,PermSet>
    where Usr: axum_login::AuthUser<Id = String>,
{
    type User = Usr;
    type Credentials = PswAuthCredentials;
    type Error = AuthBackendError;

    #[inline]
    //noinspection DuplicatedCode
    async fn authenticate(&self, creds: Self::Credentials) -> Result<Option<Self::User>, Self::Error> {
        self.psw_backend.authenticate(creds).await
    }
    #[inline]
    //noinspection DuplicatedCode
    async fn get_user(&self, user_id: &axum_login::UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        self.psw_backend.get_user(user_id).await
    }
}
#[axum::async_trait]
impl <
    Usr: axum_login::AuthUser + PswUser,
    PswComp: PasswordComparator + Debug + Clone + Sync + Send,
    Perm: Hash + Eq + Debug + Clone + Send + Sync,
    PermSet: PermissionSet<Permission=Perm> + Debug + Clone + Send + Sync
> PermissionProviderSource for HttpBasicAuthBackend<Usr,PswComp,Perm,PermSet>
    where Usr: axum_login::AuthUser<Id = String> {
    type User = Usr;
    type Permission = Perm;
    type PermissionSet = PermSet;

    #[inline]
    //noinspection DuplicatedCode
    fn permission_provider(&self) -> Arc<dyn PermissionProvider<User=Self::User, Permission=Self::Permission, PermissionSet=Self::PermissionSet>> {
        self.psw_backend.permission_provider()
    }
}
#[axum::async_trait]
impl <
    Usr: axum_login::AuthUser + PswUser,
    PswComp: PasswordComparator + Debug + Clone + Sync + Send,
    Perm: Hash + Eq + Debug + Clone + Send + Sync,
    PermSet: PermissionSet<Permission=Perm> + Debug + Clone + Send + Sync
> AuthorizeBackend for HttpBasicAuthBackend<Usr,PswComp,Perm,PermSet>
    where Usr: axum_login::AuthUser<Id = String> {
    //noinspection DuplicatedCode
}


#[axum::async_trait]
impl <
    Usr: axum_login::AuthUser + PswUser,
    PswComp: PasswordComparator + Debug + Clone + Send + Sync,
    Perm: Hash + Eq + Debug + Clone + Send + Sync,
    PermSet: PermissionSet<Permission=Perm> + Debug + Clone + Send + Sync,
> AuthnBackendAttributes for HttpBasicAuthBackend<Usr,PswComp,Perm,PermSet>
    where Usr: axum_login::AuthUser<Id = String>,
{
    type ProposeAuthAction = ProposeHttpBasicAuthAction;

    fn user_provider(&self) -> Arc<dyn AuthUserProvider<User=Usr> + Send + Sync> {
        self.psw_backend.users_provider()
    }
    fn propose_authentication_action(&self, _: &Request) -> Option<Self::ProposeAuthAction> {
        if let AuthBackendMode::AuthProposed = self.auth_mode
        { Some(ProposeHttpBasicAuthAction) } else { None }
    }
}

#[derive(Debug, Clone)]
pub struct ProposeHttpBasicAuthAction;
impl ProposeAuthAction for ProposeHttpBasicAuthAction { }
#[inherent::inherent]
impl axum::response::IntoResponse for ProposeHttpBasicAuthAction {
    #[allow(dead_code)] // !! It is really used IMPLICITLY !!
    pub fn into_response(self) -> axum::response::Response<axum::body::Body> {
        http_unauthenticated_401_response("Basic")
    }
}


#[axum::async_trait]
impl <
    Usr: axum_login::AuthUser + PswUser,
    PswComp: PasswordComparator + Debug + Clone + Send + Sync,
    Perm: Hash + Eq + Debug + Clone + Send + Sync,
    PermSet: PermissionSet<Permission=Perm> + Debug + Clone + Send + Sync,
> RequestAuthenticated for HttpBasicAuthBackend<Usr,PswComp,Perm,PermSet>
    where Usr: axum_login::AuthUser<Id = String>,
{
    async fn do_authenticate_request<S: Send + Sync>(&self, req: Request)
        -> (Request, Result<Option<Self::User>, Self::Error>)
        where Self: 'static {

        let basic_opt = req.headers().typed_get::<Authorization<Basic>>(); //"Authorization");

        let auth_res: Result<Option<Self::User>, Self::Error> =
            if let Some(Authorization(basic)) = basic_opt {
                self.authenticate(PswAuthCredentials {
                    username: basic.username().to_string(),
                    password: basic.password().to_string(),
                    next: None,
                }).await
            } else { Ok(None) };

        (req, auth_res)
    }
}


/*
#[derive(Clone)]
pub struct BasicAuthCreds(axum_extra::headers::authorization::Basic);

impl Debug for BasicAuthCreds {
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
    Usr: axum_login::AuthUser + PswUser,
    PswComp: PasswordComparator + Debug + Clone + Send + Sync,
    Perm: Hash + Eq + Debug + Clone + Send + Sync,
    PermSet: PermissionSet<Permission=Perm> + Debug + Clone + Send + Sync,
> RequestUserAuthnBackend for HttpBasicAuthBackend<Usr,PswComp,Perm,PermSet>
    where Usr: axum_login::AuthUser<Id = String>,
{
    type AuthRequestData = BasicAuthCreds;

    async fn authenticate_request_impl<S>(&self, auth_request_data: Self::AuthRequestData) -> Result<Option<Self::User>, Self::Error> {
        use axum_login::AuthnBackend;

        self.authenticate(PswAuthCredentials {
            username: auth_request_data.0.username().to_string(),
            password: auth_request_data.0.password().to_string(),
            next: None,
        }).await
    }
}
*/

/*
use super::super::auth::authn_backend_dyn_wrap::AuthnBackendDynWrapper;

impl <
    PswComparator: PasswordComparator + Clone + Send + Sync,
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
    type User: axum_login::AuthUser;
    async fn is_req_authenticated(&self, req: Request) -> (Request, Result<Option<Self::User>, AuthBackendError>);
}

#[axum::async_trait]
impl <
    Usr: axum_login::AuthUser + PswUser + 'static,
    PswComp: PasswordComparator + Debug + Clone + Send + Sync + 'static,
    Perm: Hash + Eq + Debug + Clone + Send + Sync + 'static,
    PermSet: PermissionSet<Permission=Perm> + Debug + Clone + Send + Sync + 'static,
> RequestUserAuthnBackendDyn for HttpBasicAuthBackend<Usr,PswComp,Perm,PermSet>
    where Usr: axum_login::AuthUser<Id = String>,
{
    type User = Usr;

    async fn is_req_authenticated(&self, req: Request) -> (Request, Result<Option<Self::User>, AuthBackendError>) {
        // let res = self.do_authenticate_request::<PswComp>(req).await;
        let res = self.do_authenticate_request::<()>(req).await;
        res
    }
}

use axum::extract::Request;
use axum_extra::headers::{Authorization, HeaderMapExt};
use axum_extra::headers::authorization::Basic;
use crate::auth::backend::authz_backend::{AuthorizeBackend, PermissionProviderSource};
use crate::auth::backend::RequestAuthenticated;
use crate::auth::permission::{PermissionProvider, PermissionSet};
use crate::auth::permission::empty_perm_provider::{ AlwaysAllowedPermSet, EmptyPerm };
use super::super::error::AuthBackendError;

impl <
    Usr: axum_login::AuthUser + PswUser + 'static,
    PswComp: PasswordComparator + Debug + Clone + Send + Sync + 'static,
    Perm: Hash + Eq + Debug + Clone + Send + Sync + 'static,
    PermSet: PermissionSet<Permission=Perm> + Debug + Clone + Send + Sync+ 'static,
> TryFrom<HttpBasicAuthBackend<Usr,PswComp,Perm,PermSet>>
for Arc<dyn RequestUserAuthnBackendDyn<User=Usr>>
    where Usr: axum_login::AuthUser<Id = String>,
{
    type Error = AuthBackendError;

    fn try_from(value: HttpBasicAuthBackend<Usr,PswComp,Perm,PermSet>) -> Result<Arc<dyn RequestUserAuthnBackendDyn<User=Usr>>, Self::Error> {
        let as_dyn: Arc<dyn RequestUserAuthnBackendDyn<User=Usr>> = Arc::new(value);
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
    use super::super::super::examples::auth_user::AuthUserExample as AuthUser;
    use crate::auth::AuthUserProviderError;
    use crate::auth::examples::auth_user::AuthUserExamplePswExtractor;
    use crate::auth::permission::bits_perm_set::IntegerBitsPermissionSet;
    use crate::auth::permission::empty_perm_provider::{always_allowed_perm_provider_arc, empty_always_allowed_perm_provider_arc};
    use crate::auth::permission::predefined::{ Role, RolePermissionsSet };
    use crate::util::TestResultUnwrap;

    use super::*;
    use super::super::super::{
        examples::auth_user::AuthUserExample,
        backend::{ AuthBackendMode },
        user_provider::{ AuthUserProvider, InMemAuthUserProvider },
        psw::PlainPasswordComparator,
    };
    type Perm = u32;
    type PermSet = IntegerBitsPermissionSet<u32>;

    pub fn in_memory_test_users() -> Result<InMemAuthUserProvider<AuthUser,Role,RolePermissionsSet,AuthUserExamplePswExtractor>, AuthUserProviderError> {
        InMemAuthUserProvider::with_users(vec!(AuthUser::new(1, "http-vovan", "qwerty")))
    }

    #[test]
    fn test_try_into_when_impl() {
        let users = Arc::new(in_memory_test_users().test_unwrap());
        let users: Arc<dyn AuthUserProvider<User =AuthUserExample> + Send + Sync> = users;
        let perm_provider = always_allowed_perm_provider_arc::<AuthUserExample,Perm,PermSet>();
        let basic_auth = HttpBasicAuthBackend::<AuthUserExample,PlainPasswordComparator,Perm,PermSet>::new(users, AuthBackendMode::AuthSupported, perm_provider);

        let _as_eee: Result<Arc<dyn RequestUserAuthnBackendDyn<User=AuthUserExample>>, _> =  basic_auth.try_into();
    }

    #[test]
    fn test_try_into_when_no_impl() {
        use super::super::super::backend::{ LoginFormAuthBackend, LoginFormAuthConfig };

        let users = Arc::new(in_memory_test_users().test_unwrap());
        let users: Arc<dyn AuthUserProvider<User = AuthUser> + Send + Sync> = users;
        // let users: Arc<dyn AuthUserProvider<User = AuthUser>> = users;
        let _basic_auth = LoginFormAuthBackend::<AuthUserExample, PlainPasswordComparator>::new(
            users,
            LoginFormAuthConfig { login_url: "/login", auth_mode: AuthBackendMode::AuthSupported },
            empty_always_allowed_perm_provider_arc(),
        );

        // let basic_auth_arc = Arc::new(basic_auth);
        // let _as_eee: Result<Arc<dyn RequestUserAuthnBackendDyn<User=AuthUserExample>>, _> =
        //     basic_auth_arc.try_into();
    }

    /*
    use std::sync::Arc;
    // use axum_login::AuthUser;
    use super::AuthUser;
    use super::super::auth::{AuthBackendError, AuthBackendMode, BasicAuthCreds, HttpBasicAuthBackend, InMemAuthUserProvider, PlainPasswordComparator, RequestUserAuthnBackend};
    use super::super::auth::psw_auth::PswAuthCredentials;

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