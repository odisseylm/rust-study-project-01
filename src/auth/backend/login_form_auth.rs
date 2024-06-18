use std::sync::Arc;
use axum::body::Body;
use axum::extract::OriginalUri;
use axum::http::StatusCode;

use super::{ psw_auth::PswAuthBackendImpl };
use super::super::{
    auth_backend::{ AuthBackendMode, AuthnBackendAttributes, ProposeAuthAction },
    auth_user_provider::AuthUserProvider,
    psw::PasswordComparator,
    util::http::url_encode,
};


#[derive(Debug, Clone)]
pub struct LoginFormAuthConfig {
    pub auth_mode: AuthBackendMode,
    pub login_url: &'static str,
}


use axum_login::AuthnBackend;
use crate::auth::backend::psw_auth::PswUser;
// use super::axum_login_delegatable::ambassador_impl_AuthnBackend;

#[derive(Clone)]
// #[derive(Clone, ambassador::Delegate)]
#[readonly::make] // should be after 'derive'
// #[delegate(axum_login::AuthnBackend, target = "psw_backend")]
pub struct LoginFormAuthBackend <
    User: axum_login::AuthUser + PswUser,
    PswComparator: PasswordComparator + Clone + Sync + Send,
> {
    psw_backend: PswAuthBackendImpl<User,PswComparator>,
    pub config: LoginFormAuthConfig,
}

impl <
    User: axum_login::AuthUser + PswUser,
    PswComparator: PasswordComparator + Clone + Sync + Send,
> LoginFormAuthBackend<User,PswComparator> {
    pub fn new(
        users_provider: Arc<dyn AuthUserProvider<User=User> + Sync + Send>,
        config: LoginFormAuthConfig,
    ) -> LoginFormAuthBackend<User,PswComparator> {
        LoginFormAuthBackend::<User,PswComparator> {
            // psw_backend: PswAuthBackendImpl::<User,PswComparator>::new(users_provider.clone()),
            psw_backend: PswAuthBackendImpl::new(users_provider.clone()),
            config,
        }
    }
}

/*
#[axum::async_trait]
impl <
    PswComparator: PasswordComparator + Clone + Sync + Send,
> core::borrow::Borrow<dyn axum_login::AuthnBackend<User=AuthUser,Credentials=PswAuthCredentials,Error=AuthBackendError>>
for LoginFormAuthBackend<PswComparator> {
    fn borrow(&self) -> &dyn axum_login::AuthnBackend<User=AuthUser,Credentials=PswAuthCredentials,Error=AuthBackendError> {
        &self.psw_backend
    }
}
*/
/*
#[axum::async_trait]
impl <
    PswComparator: PasswordComparator + Clone + Sync + Send,
> core::ops::Deref for LoginFormAuthBackend<PswComparator> {
    type Target = dyn axum_login::AuthnBackend<User=AuthUser,Credentials=PswAuthCredentials,Error=AuthBackendError>;
    // type Target = dyn axum_login::AuthnBackend;

    fn deref(&self) -> &Self::Target {
        &self.psw_backend
    }
}
*/


// T O D O: how to avoid duplicating this code?
//       Deref/Borrow do not work because they use 'dyn' and axum_login::AuthnBackend
//       requires Clone which can NOT be used with as 'dyn'.
#[axum::async_trait]
impl <
    User: axum_login::AuthUser + PswUser,
    PswComparator: PasswordComparator + Clone + Sync + Send,
> AuthnBackend for LoginFormAuthBackend<User,PswComparator>
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
    PswComparator: PasswordComparator + Clone + Sync + Send,
> AuthnBackendAttributes for LoginFormAuthBackend<User,PswComparator>
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
