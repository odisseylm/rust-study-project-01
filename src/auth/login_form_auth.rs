use std::sync::Arc;
use axum::body::Body;
use axum::extract::OriginalUri;
use axum::http::StatusCode;
use psw_auth::PswAuthCredentials;
use crate::auth::auth_backend::{AuthnBackendAttributes, ProposeAuthAction};
use crate::auth::http::url_encode;

use super::psw_auth;
use super::psw_auth::PswAuthBackendImpl;
use super::error::AuthBackendError;
use super::auth_user_provider::AuthUserProvider;
use super::auth_user::AuthUser;
use super::psw::PasswordComparator;


// pub type LoginFormAuthAuthSession <
//     PswComparator, // : PasswordComparator + Clone + Sync + Send,
// > = axum_login::AuthSession<LoginFormAuthBackend<PswComparator>>;


#[derive(Copy, Clone, Debug)]
pub enum LoginFormAuthMode { // TODO: probably we can remove it, or replace with something more abstract ???
    LoginFormAuthSupported,
    LoginFormAuthProposed { login_form_url: Option<&'static str> },
}


#[derive(Clone)]
#[readonly::make]
pub struct LoginFormAuthBackend <
    PswComparator: PasswordComparator + Clone + Sync + Send,
> {
    psw_backend: PswAuthBackendImpl<PswComparator>,
    pub login_from_auth_mode: LoginFormAuthMode,
}

impl <
    PswComparator: PasswordComparator + Clone + Sync + Send,
> LoginFormAuthBackend<PswComparator> {
    pub fn new(
        users_provider: Arc<dyn AuthUserProvider<User = AuthUser> + Sync + Send>,
        login_from_auth_mode: LoginFormAuthMode,
    ) -> LoginFormAuthBackend<PswComparator> {
        LoginFormAuthBackend::<PswComparator> {
            psw_backend: PswAuthBackendImpl::new(users_provider.clone()),
            login_from_auth_mode,
        }
    }
}

// TODO: how to avoid duplicating this code? (probably Deref or something like that)
#[axum::async_trait]
impl <
    PswComparator: PasswordComparator + Clone + Sync + Send,
> axum_login::AuthnBackend for LoginFormAuthBackend<PswComparator> {
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
    async fn get_user(&self, user_id: &axum_login::UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        self.psw_backend.get_user(user_id).await
    }
}


#[axum::async_trait]
impl <
    PswComparator: PasswordComparator + Clone + Sync + Send,
    > AuthnBackendAttributes for LoginFormAuthBackend<PswComparator> {
    type ProposeAuthAction = ProposeLoginFormAuthAction;

    fn user_provider(&self) -> Arc<dyn AuthUserProvider<User=AuthUser> + Sync + Send> {
        self.psw_backend.users_provider()
    }
    fn propose_authentication_action(&self, req: &axum::extract::Request) -> Option<Self::ProposeAuthAction> {
        if let LoginFormAuthMode::LoginFormAuthProposed { login_form_url } = self.login_from_auth_mode {
            let initial_uri: Option<String> = req.extensions().get::<OriginalUri>().map(|uri|uri.to_string());
            Some(ProposeLoginFormAuthAction { login_form_url, initial_url: initial_uri })
        } else { None }
    }
}

pub struct ProposeLoginFormAuthAction {
    pub login_form_url: Option<&'static str>,
    pub initial_url: Option<String>,
}
impl ProposeAuthAction for ProposeLoginFormAuthAction { }
#[inherent::inherent]
impl axum::response::IntoResponse for ProposeLoginFormAuthAction {
    #[allow(dead_code)] // !! It is really used IMPLICITLY !!
    pub fn into_response(self) -> axum::response::Response<Body> {
        let login_url = self.login_form_url.unwrap_or("/login");
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
