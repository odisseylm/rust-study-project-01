use std::fmt::Debug;
use std::hash::Hash;
use std::sync::Arc;
use axum::body::Body;
use axum::extract::OriginalUri;
use axum::http::StatusCode;

use super::{ psw_auth::PswAuthBackendImpl };
use super::super::{
    backend::{ AuthBackendMode, AuthnBackendAttributes, ProposeAuthAction },
    user_provider::AuthUserProvider,
    psw::PasswordComparator,
    util::http::url_encode,
};


#[derive(Debug, Clone)]
pub struct LoginFormAuthConfig {
    pub auth_mode: AuthBackendMode,
    pub login_url: &'static str,
}


use axum_login::AuthnBackend;
use crate::auth::backend::authz_backend::{AuthorizeBackend, PermissionProviderSource};
use crate::auth::backend::psw_auth::PswUser;
use crate::auth::permission::{PermissionProvider, PermissionSet};
use crate::auth::permission::empty_permission_provider::{ AlwaysAllowedPermSet, EmptyPerm };
// use super::axum_login_delegatable::ambassador_impl_AuthnBackend;

#[derive(Clone)]
// #[derive(Clone, ambassador::Delegate)]
#[readonly::make] // should be after 'derive'
// #[delegate(axum_login::AuthnBackend, target = "psw_backend")]
pub struct LoginFormAuthBackend <
    User: axum_login::AuthUser + PswUser,
    PswComparator: PasswordComparator + Debug + Clone + Send + Sync,
    Perm: Hash + Eq + Debug + Clone + Send + Sync = EmptyPerm,
    PermSet: PermissionSet<Permission=Perm> + Debug + Clone + Send + Sync = AlwaysAllowedPermSet<Perm>,
> {
    psw_backend: PswAuthBackendImpl<User,PswComparator,Perm,PermSet>,
    pub config: LoginFormAuthConfig,
}

impl <
    User: axum_login::AuthUser + PswUser,
    PswComparator: PasswordComparator + Debug + Clone + Send + Sync,
    Perm: Hash + Eq + Debug + Clone + Send + Sync,
    PermSet: PermissionSet<Permission=Perm> + Debug + Clone + Send + Sync,
> LoginFormAuthBackend<User,PswComparator,Perm,PermSet> {
    pub fn new(
        users_provider: Arc<dyn AuthUserProvider<User=User> + Sync + Send>,
        config: LoginFormAuthConfig,
        permission_provider: Arc<dyn PermissionProvider<User=User,Permission=Perm,PermissionSet=PermSet> + Sync + Send>,
    ) -> LoginFormAuthBackend<User,PswComparator,Perm,PermSet> {
        LoginFormAuthBackend::<User,PswComparator,Perm,PermSet> {
            psw_backend: PswAuthBackendImpl::new(
                users_provider,
                permission_provider,
            ),
            config,
        }
    }
}


// T O D O: how to avoid duplicating this code?
//       Deref/Borrow do not work because they use 'dyn' and axum_login::AuthnBackend
//       requires Clone which can NOT be used with as 'dyn'.
#[axum::async_trait]
impl <
    User: axum_login::AuthUser + PswUser,
    PswComparator: PasswordComparator + Debug + Clone + Send + Sync,
    Perm: Hash + Eq + Debug + Clone + Send + Sync,
    PermSet: PermissionSet<Permission=Perm> + Debug + Clone + Send + Sync,
> AuthnBackend for LoginFormAuthBackend<User,PswComparator,Perm,PermSet>
    where User: axum_login::AuthUser<Id = String>,
{
    type User = User;
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
#[axum::async_trait]
impl <
    User: axum_login::AuthUser + PswUser,
    PswComparator: PasswordComparator + Debug + Clone + Sync + Send,
    Perm: Hash + Eq + Debug + Clone + Send + Sync,
    PermSet: PermissionSet<Permission=Perm> + Debug + Clone + Send + Sync
> PermissionProviderSource for LoginFormAuthBackend<User,PswComparator,Perm,PermSet>
    where User: axum_login::AuthUser<Id = String> {
    type User = User;
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
    User: axum_login::AuthUser + PswUser,
    PswComparator: PasswordComparator + Debug + Clone + Sync + Send,
    Perm: Hash + Eq + Debug + Clone + Send + Sync,
    PermSet: PermissionSet<Permission=Perm> + Debug + Clone + Send + Sync
> AuthorizeBackend for LoginFormAuthBackend<User,PswComparator,Perm,PermSet>
    where User: axum_login::AuthUser<Id = String> {
    //noinspection DuplicatedCode
}


#[axum::async_trait]
impl <
    User: axum_login::AuthUser + PswUser,
    PswComparator: PasswordComparator + Debug + Clone + Send + Sync,
    Perm: Hash + Eq + Debug + Clone + Send + Sync,
    PermSet: PermissionSet<Permission=Perm> + Debug + Clone + Send + Sync,
> AuthnBackendAttributes for LoginFormAuthBackend<User,PswComparator,Perm,PermSet>
    where User: axum_login::AuthUser<Id = String>,
{
    type ProposeAuthAction = ProposeLoginFormAuthAction;

    fn user_provider(&self) -> Arc<dyn AuthUserProvider<User=User> + Sync + Send> {
        self.psw_backend.users_provider()
    }
    fn propose_authentication_action(&self, req: &axum::extract::Request) -> Option<Self::ProposeAuthAction> {
        if let AuthBackendMode::AuthProposed = self.config.auth_mode {
            let initial_url: Option<String> = req.extensions().get::<OriginalUri>().map(|uri|uri.to_string());
            Some(ProposeLoginFormAuthAction { login_url: Some(self.config.login_url), initial_url })
        } else { None }
    }
}

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
            Some(ref initial_url) => format!("{}?next={}", login_url, url_encode(initial_url.as_str())),
        };
        axum::response::Response::builder()
            .status(StatusCode::UNAUTHORIZED) // redirect
            .header("Location", login_url.clone())
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
