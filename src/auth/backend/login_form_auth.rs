use core::fmt::Debug;
use core::hash::Hash;
use std::sync::Arc;
use axum::body::Body;
use axum::extract::OriginalUri;
use axum::http::StatusCode;

use super::{ psw_auth::PswAuthBackendImpl, RequestAuthenticated };
use super::super::{
    backend::{
        AuthBackendMode, AuthnBackendAttributes, ProposeAuthAction,
        psw_auth::PswUser,
        authz_backend::{ AuthorizeBackend, PermissionProviderSource },
    },
    user_provider::AuthUserProvider,
    psw::PasswordComparator,
    permission::{
        PermissionProvider, PermissionSet,
        empty_perm_provider::{ AlwaysAllowedPermSet, EmptyPerm },
    },
    util::http::url_encode,
};

#[cfg(feature = "ambassador")]
use axum_login::AuthnBackend;
#[cfg(feature = "ambassador")]
use super::{
    super::{
        backend::authz_backend:: {
            AuthorizationResult,
            authz_backend::{
                ambassador_impl_PermissionProviderSource,
                ambassador_impl_AuthorizeBackend,
            },
        },
        permission::PermissionProcessError,
    },
    axum_login_delegatable::ambassador_impl_AuthnBackend,
};

// -------------------------------------------------------------------------------------------------



#[derive(Debug, Clone)]
pub struct LoginFormAuthConfig {
    pub auth_mode: AuthBackendMode,
    pub login_url: &'static str,
}


#[derive(Clone)]
#[cfg_attr(feature = "ambassador", derive(ambassador::Delegate))]
#[readonly::make] // should be after 'derive'
#[cfg_attr(feature = "ambassador", delegate(axum_login::AuthnBackend, target = "psw_backend"))]
#[cfg_attr(feature = "ambassador", delegate(PermissionProviderSource, target = "psw_backend"))]
#[cfg_attr(feature = "ambassador", delegate(AuthorizeBackend, target = "psw_backend"))]
pub struct LoginFormAuthBackend <
    User: Clone,
    PswComparator: Clone,
    Perm: Clone = EmptyPerm,
    PermSet: Clone = AlwaysAllowedPermSet<Perm>,
> {
    psw_backend: PswAuthBackendImpl<User,PswComparator,Perm,PermSet>,
    pub config: LoginFormAuthConfig,
}


impl <
    Usr: axum_login::AuthUser,
    PswComp: PasswordComparator + Clone,
    Perm: Hash + Eq + Debug + Clone + Send + Sync,
    PermSet: PermissionSet<Permission=Perm> + Clone,
> LoginFormAuthBackend<Usr,PswComp,Perm,PermSet> {
    pub fn new(
        users_provider: Arc<dyn AuthUserProvider<User=Usr> + Sync + Send>,
        config: LoginFormAuthConfig,
        permission_provider: Arc<dyn PermissionProvider<User=Usr,Permission=Perm,PermissionSet=PermSet> + Sync + Send>,
    ) -> LoginFormAuthBackend<Usr,PswComp,Perm,PermSet> {
        LoginFormAuthBackend::<Usr,PswComp,Perm,PermSet> {
            psw_backend: PswAuthBackendImpl::new(
                users_provider,
                permission_provider,
            ),
            config,
        }
    }
}


#[cfg(not(feature = "ambassador"))]
// How to avoid duplicating this code?
// Deref/Borrow do not work because they use 'dyn' and axum_login::AuthnBackend
// requires Clone which can NOT be used with as 'dyn'.
//
#[axum::async_trait]
impl <
    Usr: axum_login::AuthUser + PswUser,
    PswComp: PasswordComparator + Debug + Clone + Send + Sync,
    Perm: Hash + Eq + Debug + Clone + Send + Sync,
    PermSet: PermissionSet<Permission=Perm> + Clone,
> axum_login::AuthnBackend for LoginFormAuthBackend<Usr,PswComp,Perm,PermSet>
    where Usr: axum_login::AuthUser<Id = String>,
{
    type User = Usr;
    type Credentials = super::psw_auth::PswAuthCredentials;
    type Error = super::super::error::AuthBackendError;

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

#[cfg(not(feature = "ambassador"))]
#[axum::async_trait]
impl <
    Usr: axum_login::AuthUser + PswUser,
    PswComp: PasswordComparator + Debug + Clone + Sync + Send,
    Perm: Hash + Eq + Debug + Clone + Send + Sync,
    PermSet: PermissionSet<Permission=Perm> + Clone,
> PermissionProviderSource for LoginFormAuthBackend<Usr,PswComp,Perm,PermSet>
    where Usr: axum_login::AuthUser<Id = String> {
    type User = Usr;
    type Permission = Perm;
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

#[cfg(not(feature = "ambassador"))]
#[axum::async_trait]
impl <
    Usr: axum_login::AuthUser + PswUser,
    PswComp: PasswordComparator + Debug + Clone + Sync + Send,
    Perm: Hash + Eq + Debug + Clone + Send + Sync,
    PermSet: PermissionSet<Permission=Perm> + Clone,
> AuthorizeBackend for LoginFormAuthBackend<Usr,PswComp,Perm,PermSet>
    where Usr: axum_login::AuthUser<Id = String> {
    //noinspection DuplicatedCode
}

#[axum::async_trait]
impl <
    Usr: axum_login::AuthUser + PswUser,
    PswComp: PasswordComparator + Debug + Clone + Send + Sync,
    Perm: Hash + Eq + Debug + Clone + Send + Sync,
    PermSet: PermissionSet<Permission=Perm> + Clone,
> RequestAuthenticated for LoginFormAuthBackend<Usr,PswComp,Perm,PermSet>
    where Usr: axum_login::AuthUser<Id = String>,
{ }


#[axum::async_trait]
impl <
    Usr: axum_login::AuthUser + PswUser,
    PswComp: PasswordComparator + Debug + Clone + Send + Sync,
    Perm: Hash + Eq + Debug + Clone + Send + Sync,
    PermSet: PermissionSet<Permission=Perm> + Clone,
> AuthnBackendAttributes for LoginFormAuthBackend<Usr,PswComp,Perm,PermSet>
    where Usr: axum_login::AuthUser<Id = String>,
{
    type ProposeAuthAction = ProposeLoginFormAuthAction;

    fn user_provider(&self) -> Arc<dyn AuthUserProvider<User=Usr> + Sync + Send> {
        self.psw_backend.users_provider()
    }
    fn user_provider_ref<'a>(&'a self) -> &'a Arc<dyn AuthUserProvider<User=Self::User> + Sync + Send> {
        &self.psw_backend.users_provider
    }

    fn propose_authentication_action(&self, req: &axum::extract::Request) -> Option<Self::ProposeAuthAction> {
        if let AuthBackendMode::AuthProposed = self.config.auth_mode {
            let initial_url: Option<String> = req.extensions().get::<OriginalUri>().map(|uri|uri.to_string());
            Some(ProposeLoginFormAuthAction { login_url: Some(self.config.login_url), initial_url })
        } else { None }
    }
}

#[derive(Debug, Clone)]
pub struct ProposeLoginFormAuthAction {
    pub login_url: Option<&'static str>,
    pub initial_url: Option<String>,
}

impl ProposeAuthAction for ProposeLoginFormAuthAction { }
#[inherent::inherent]
impl axum::response::IntoResponse for ProposeLoginFormAuthAction {

    #[allow(dead_code)] // !! It is really used IMPLICITLY !!
    pub fn into_response(self) -> axum::response::Response<Body> {
        let login_url = self.login_url.unwrap_or("/login");
        let login_url = match self.initial_url {
            None => login_url.to_string(),
            Some(ref initial_url) => format!("{login_url}?next={}", url_encode(initial_url.as_str())),
        };

        axum::response::Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .header("Location", login_url.clone()) // redirect
            .header("Content-Type", "text/html; charset=utf-8")
            .body(Body::from(REDIRECT_LOGIN_PAGE_CONTENT.replace("{login_url}", login_url.as_str())))
            .unwrap_or_else(|_err| StatusCode::UNAUTHORIZED.into_response())

    }
}


static REDIRECT_LOGIN_PAGE_CONTENT: &'static str = r#"
<!doctype html>
<html>
  <head>
    <meta charset="utf-8" />
    <meta http-equiv="refresh" content="0; url={login_url}">
    <title>User is not athenticated</title>
  </head>
  <body>
    <a href="{login_url}">Login</a>
  </body>
</html>
"#;
