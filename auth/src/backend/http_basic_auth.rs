use core::fmt::Debug;
use std::sync::Arc;
use axum::extract::Request;
use axum_extra::headers::{ Authorization, HeaderMapExt, authorization::Basic };

use super::psw_auth::{ PswAuthBackendImpl, PswAuthCredentials, PswUser };
use crate::{
    util::http::http_unauthenticated_401_response,
    backend::{
        AuthBackendMode, AuthnBackendAttributes, ProposeAuthAction, RequestAuthenticated,
        authz_backend::{ PermissionProviderSource, AuthorizeBackend },
    },
    user_provider::AuthUserProvider,
    psw::PasswordComparator,
    permission::{
        PermissionSet, PermissionProvider,
        empty_perm_provider::{ EmptyPerm, AlwaysAllowedPermSet },
    },
};

// #[cfg(feature = "ambassador")]
use axum_login::AuthnBackend;
#[cfg(feature = "ambassador")]
use crate::backend::{
    axum_login_delegatable::ambassador_impl_AuthnBackend,
};
// -------------------------------------------------------------------------------------------------


#[derive(Clone)]
#[cfg_attr(feature = "ambassador", derive(ambassador::Delegate))]
#[readonly::make] // should be after 'derive'
#[cfg_attr(feature = "ambassador", delegate(axum_login::AuthnBackend, target = "psw_backend"))]
pub struct HttpBasicAuthBackend <
    User: axum_login::AuthUser + PswUser,
    PswComparator: PasswordComparator + Debug + Clone + Send + Sync,
    PermSet: PermissionSet + Clone = AlwaysAllowedPermSet<EmptyPerm>,
> where User: axum_login::AuthUser<Id = String> {
    psw_backend: PswAuthBackendImpl<User,PswComparator,PermSet>,
    pub auth_mode: AuthBackendMode,
}


impl <
    Usr: axum_login::AuthUser + PswUser,
    PswComp: PasswordComparator + Debug + Clone + Send + Sync,
    PermSet: PermissionSet + Clone,
> HttpBasicAuthBackend<Usr,PswComp,PermSet>
    where Usr: axum_login::AuthUser<Id = String>,
{
    pub fn new(
        users_provider: Arc<dyn AuthUserProvider<User=Usr> + Send + Sync>,
        auth_mode: AuthBackendMode,
        permission_provider: Arc<dyn PermissionProvider<User=Usr,Permission=<PermSet as PermissionSet>::Permission,PermissionSet=PermSet> + Send + Sync>,
    ) -> HttpBasicAuthBackend<Usr,PswComp,PermSet> {
        HttpBasicAuthBackend::<Usr,PswComp,PermSet> {
            psw_backend: PswAuthBackendImpl::new(users_provider, permission_provider),
            auth_mode,
        }
    }

    async fn do_authenticate_impl <
        RootBackend: AuthnBackend + 'static,
        S: Send + Sync,
    > (&self, headers: &http::HeaderMap)
      -> Result<Option<Usr>, crate::error::AuthBackendError>
    where Self: 'static {
        let basic_opt = headers.typed_get::<Authorization<Basic>>(); //"Authorization");

        let auth_res: Result<Option<Usr>, crate::error::AuthBackendError> =
            if let Some(Authorization(basic)) = basic_opt {
                use axum_login::AuthnBackend;
                self.authenticate(PswAuthCredentials {
                    username: basic.username().to_string(),
                    password: basic.password().to_string(),
                    next: None,
                }).await
            } else { Ok(None) };

        auth_res
    }

}


#[cfg(not(feature = "ambassador"))]
#[axum::async_trait]
impl <
    Usr: axum_login::AuthUser + PswUser,
    Perm: Hash + Eq + Debug + Clone + Send + Sync,
    PermSet: PermissionSet + Clone,
> axum_login::AuthnBackend for HttpBasicAuthBackend<Usr,PswComp,PermSet>
    where Usr: axum_login::AuthUser<Id = String>,
{
    type User = Usr;
    type Credentials = PswAuthCredentials;
    type Error = crate::error::AuthBackendError;

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


// #[cfg(not(feature = "ambassador"))]
#[axum::async_trait]
impl <
    Usr: axum_login::AuthUser + PswUser,
    PswComp: PasswordComparator + Debug + Clone + Send + Sync,
    PermSet: PermissionSet + Clone,
> PermissionProviderSource for HttpBasicAuthBackend<Usr,PswComp,PermSet>
    where
        Usr: axum_login::AuthUser<Id = String> {
    type User = Usr;
    type Permission = <PermSet as PermissionSet>::Permission;
    type PermissionSet = PermSet;

    #[inline]
    //noinspection DuplicatedCode
    fn permission_provider(&self) -> Arc<dyn PermissionProvider<User=Self::User, Permission=Self::Permission, PermissionSet=Self::PermissionSet> + Send + Sync> {
        self.psw_backend.permission_provider()
    }
    #[inline]
    //noinspection DuplicatedCode
    fn permission_provider_ref<'a>(&'a self) -> &'a Arc<dyn PermissionProvider<User=Self::User, Permission=Self::Permission, PermissionSet=Self::PermissionSet> + Send + Sync> {
        &self.psw_backend.permission_provider
    }
}


// #[cfg(not(feature = "ambassador"))] // not supported by 'ambassador' now since it is not delegation
#[axum::async_trait]
impl <
    Usr: axum_login::AuthUser + PswUser,
    PswComp: PasswordComparator + Debug + Clone + Send + Sync,
    PermSet: PermissionSet + Clone,
> AuthorizeBackend for HttpBasicAuthBackend<Usr,PswComp,PermSet>
    where Usr: axum_login::AuthUser<Id = String> {
    //noinspection DuplicatedCode
}


#[axum::async_trait]
impl <
    Usr: axum_login::AuthUser + PswUser,
    PswComp: PasswordComparator + Debug + Clone + Send + Sync,
    PermSet: PermissionSet + Clone,
> AuthnBackendAttributes for HttpBasicAuthBackend<Usr,PswComp,PermSet>
    where Usr: axum_login::AuthUser<Id = String>,
{
    type ProposeAuthAction = ProposeHttpBasicAuthAction;

    fn user_provider(&self) -> Arc<dyn AuthUserProvider<User=Usr> + Send + Sync> {
        self.psw_backend.users_provider()
    }

    fn user_provider_ref<'a>(&'a self) -> &'a Arc<dyn AuthUserProvider<User=Self::User> + Sync + Send> {
        &self.psw_backend.users_provider
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
    PermSet: PermissionSet + Clone,
> RequestAuthenticated for HttpBasicAuthBackend<Usr,PswComp,PermSet>
    where Usr: axum_login::AuthUser<Id = String>,
{
    async fn do_authenticate_request <
        RootBackend: AuthnBackend + 'static,
        S: Send + Sync,
    > (&self, _auth_session: axum_login::AuthSession<RootBackend>, req: Request)
    -> (Request, Result<Option<Self::User>, Self::Error>)
    where Self: 'static
    {
        let auth_res = self.do_authenticate_impl::<RootBackend, S>(req.headers()).await;
        (req, auth_res)
    }

    async fn do_authenticate_request_parts <
        RootBackend: AuthnBackend + 'static,
        S: Send + Sync,
    > (&self, _auth_session: axum_login::AuthSession<RootBackend>, req: &http::request::Parts)
    -> Result<Option<Self::User>, Self::Error>
    where Self: 'static {
        self.do_authenticate_impl::<RootBackend, S>(&req.headers).await
    }

}


#[cfg(test)]
mod tests {
    // use super::*;
    // use super::investigation::*;
    use std::sync::Arc;
    use crate::{
        AuthUserProviderError,
        examples::auth_user::{ AuthUserExample, AuthUserExamplePswExtractor },
        permission::{
            empty_perm_provider::{ empty_always_allowed_perm_provider_arc },
            predefined::{Role, RolePermissionsSet},
        },
        backend::{ AuthBackendMode },
        user_provider::{ AuthUserProvider, InMemAuthUserProvider },
        psw::PlainPasswordComparator,
    };
    use crate::test::TestResultUnwrap;

    // type PermSet = IntegerBitsPermissionSet<u32>;

    pub fn in_memory_test_users() -> Result<InMemAuthUserProvider<AuthUserExample,Role,RolePermissionsSet,AuthUserExamplePswExtractor>, AuthUserProviderError> {
        InMemAuthUserProvider::with_users([AuthUserExample::new(1, "http-vovan", "qwerty")])
    }

    /*
    #[test]
    fn test_try_into_when_impl() {
        let users = Arc::new(in_memory_test_users().test_unwrap());
        let users: Arc<dyn AuthUserProvider<User =AuthUserExample> + Send + Sync> = users;
        let perm_provider = always_allowed_perm_provider_arc::<AuthUserExample,PermSet>();
        let basic_auth = HttpBasicAuthBackend::<AuthUserExample,PlainPasswordComparator,Perm,PermSet>::new(users, AuthBackendMode::AuthSupported, perm_provider);

        let _as_eee: Result<Arc<dyn RequestUserAuthnBackendDyn<User=AuthUserExample>>, _> =  basic_auth.try_into();
    }
    */

    #[test]
    fn test_try_into_when_no_impl() {
        use crate::backend::{ LoginFormAuthBackend, LoginFormAuthConfig };

        let users = Arc::new(in_memory_test_users().test_unwrap());
        let users: Arc<dyn AuthUserProvider<User=AuthUserExample> + Send + Sync> = users;
        let _basic_auth = LoginFormAuthBackend::<AuthUserExample, PlainPasswordComparator>::new(
            users,
            LoginFormAuthConfig { login_url: "/test_login", auth_mode: AuthBackendMode::AuthSupported },
            empty_always_allowed_perm_provider_arc(),
        );

        // let basic_auth_arc = Arc::new(basic_auth);
        // let _as_eee: Result<Arc<dyn RequestUserAuthnBackendDyn<User=AuthUserExample>>, _> =
        //     basic_auth_arc.try_into();
    }

}