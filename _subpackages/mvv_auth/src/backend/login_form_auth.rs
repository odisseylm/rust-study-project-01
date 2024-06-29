use core::fmt::Debug;
use core::hash::Hash;
use std::sync::Arc;
use axum::body::Body;
use axum::extract::OriginalUri;
use axum::http::StatusCode;

use super::{ psw_auth::PswAuthBackendImpl, RequestAuthenticated };
use crate::{
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
use crate::backend::{
    axum_login_delegatable::ambassador_impl_AuthnBackend,
};
#[cfg(feature = "ambassador22")]
use super::{
    super::{
        backend::authz_backend::{
            AuthorizationResult,
            ambassador_impl_PermissionProviderSource,
            ambassador_impl_AuthorizeBackend,
        },
        permission::PermissionProcessError,
    },
};

// -------------------------------------------------------------------------------------------------



#[derive(Debug, Clone)]
pub struct LoginFormAuthConfig {
    pub auth_mode: AuthBackendMode,
    pub login_url: &'static str,
}

impl Default for LoginFormAuthConfig {
    fn default() -> Self {
        LoginFormAuthConfig {
            auth_mode: AuthBackendMode::AuthProposed,
            login_url: "/login",
        }
    }
}


#[derive(Clone)]
#[cfg_attr(feature = "ambassador", derive(ambassador::Delegate))]
#[readonly::make] // should be after 'derive'
#[cfg_attr(feature = "ambassador", delegate(axum_login::AuthnBackend, target = "psw_backend"))]
// #[cfg_attr(feature = "ambassador", delegate(PermissionProviderSource, target = "psw_backend"))]
// #[cfg_attr(feature = "ambassador", delegate(AuthorizeBackend, target = "psw_backend"))]
pub struct LoginFormAuthBackend <
    User: axum_login::AuthUser + PswUser,
    PswComparator: PasswordComparator + Debug + Clone + Send + Sync,
    PermSet: PermissionSet + Clone = AlwaysAllowedPermSet<EmptyPerm>,
>   where
        <PermSet as PermissionSet>::Permission : Hash + Eq,
{
    psw_backend: PswAuthBackendImpl<User,PswComparator,PermSet>,
    pub config: LoginFormAuthConfig,
}


impl <
    Usr: axum_login::AuthUser + PswUser,
    PswComp: PasswordComparator + Debug + Clone + Send + Sync,
    PermSet: PermissionSet + Clone,
> LoginFormAuthBackend<Usr,PswComp,PermSet>
    where
        <PermSet as PermissionSet>::Permission : Hash + Eq,
{
    pub fn new(
        users_provider: Arc<dyn AuthUserProvider<User=Usr> + Sync + Send>,
        config: LoginFormAuthConfig,
        permission_provider: Arc<dyn PermissionProvider<User=Usr,Permission=<PermSet as PermissionSet>::Permission,PermissionSet=PermSet> + Sync + Send>,
    ) -> LoginFormAuthBackend<Usr,PswComp,PermSet> {
        LoginFormAuthBackend::<Usr,PswComp,PermSet> {
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
    PermSet: PermissionSet<Permission=Perm> + Clone,
> axum_login::AuthnBackend for LoginFormAuthBackend<Usr, PswComp, PermSet>
    where Usr: axum_login::AuthUser<Id = String>,
{
    type User = Usr;
    type Credentials = super::psw_auth::PswAuthCredentials;
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
> PermissionProviderSource for LoginFormAuthBackend<Usr,PswComp,PermSet>
    where
        <PermSet as PermissionSet>::Permission : Hash + Eq,
        Usr: axum_login::AuthUser<Id = String>,
{
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

// #[cfg(not(feature = "ambassador"))]
#[axum::async_trait]
impl <
    Usr: axum_login::AuthUser + PswUser,
    PswComp: PasswordComparator + Debug + Clone + Send + Sync,
    PermSet: PermissionSet + Clone,
> AuthorizeBackend for LoginFormAuthBackend<Usr,PswComp,PermSet>
    where
        <PermSet as PermissionSet>::Permission : Hash + Eq,
        Usr: axum_login::AuthUser<Id = String>,
{ }

#[axum::async_trait]
impl <
    Usr: axum_login::AuthUser + PswUser,
    PswComp: PasswordComparator + Debug + Clone + Send + Sync,
    PermSet: PermissionSet + Clone,
> RequestAuthenticated for LoginFormAuthBackend<Usr,PswComp,PermSet>
    where
        <PermSet as PermissionSet>::Permission : Hash + Eq,
        Usr: axum_login::AuthUser<Id = String>,
{ }


#[axum::async_trait]
impl <
    Usr: axum_login::AuthUser + PswUser,
    PswComp: PasswordComparator + Debug + Clone + Send + Sync,
    PermSet: PermissionSet + Clone,
> AuthnBackendAttributes for LoginFormAuthBackend<Usr,PswComp,PermSet>
    where
        <PermSet as PermissionSet>::Permission : Hash + Eq,
        Usr: axum_login::AuthUser<Id = String>,
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

# [derive(Debug, Clone)]
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


static REDIRECT_LOGIN_PAGE_CONTENT: & 'static str = r#"
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


pub mod web {
    use std::fmt::Debug;
    use core::hash::Hash;
    use askama::Template;
    use axum::{
        extract::Query,
        http::StatusCode,
        response::{IntoResponse, Redirect},
        routing::{get as GET, post as POST},
        Form, Router,
    };
    // use axum_login::tower_sessions::Session;
    use serde::Deserialize;
    use crate::backend::psw_auth::PswUser;
    use crate::PasswordComparator;
    use crate::permission::PermissionSet;


    pub const NEXT_URL_KEY: &str = "auth.next-url";

    #[derive(Template)]
    #[template(path = "mvv_auth/login_form/login.html")]
    pub struct LoginTemplate {
        pub message: Option<String>,
        pub next: Option<String>,
    }


    // This allows us to extract the "next" field from the query string. We use this
    // to redirect after log in.
    #[derive(Debug, Deserialize)]
    pub struct NextUrl {
        next: Option<String>,
    }

    pub fn login_router <
        User: axum_login::AuthUser + PswUser + 'static,
        PswComparator: PasswordComparator + Debug + Clone + Send + Sync + 'static,
        // !!! We cannot use there default params (like EmptyPerm/AlwaysAllowedPermSet) because axum_login
        // uses type_id for looking data in the session.
        Perm: Hash + Eq + Debug + Clone + Send + Sync+ 'static,
        PermSet: PermissionSet<Permission=Perm> + Clone+ 'static,
    > () -> Router<()>
        where User: axum_login::AuthUser<Id = String>,
    {
        Router::new()
            .route("/login", POST(post::login::<User,PswComparator,Perm,PermSet>))
            .route("/login", GET(get::login))
            .route("/logout", GET(get::logout::<User,PswComparator,Perm,PermSet>))
    }

    mod post {
        use super::*;

        use core::fmt::Debug;
        use core::hash::Hash;
        use log::error;
        use crate::{ PasswordComparator };
        use crate::backend::{ LoginFormAuthBackend, PswAuthCredentials, psw_auth::PswUser };
        use crate::permission::PermissionSet;

        pub async fn login <
            User: axum_login::AuthUser + PswUser,
            PswComparator: PasswordComparator + Debug + Clone + Send + Sync,
            // !!! We cannot use there default params (like EmptyPerm/AlwaysAllowedPermSet) because axum_login
            // uses type_id for looking data in the session.
            Perm: Hash + Eq + Debug + Clone + Send + Sync,
            PermSet: PermissionSet<Permission=Perm> + Clone,
        > (
            mut auth_session: axum_login::AuthSession<LoginFormAuthBackend<User,PswComparator,PermSet>>,
            Form(creds): Form<PswAuthCredentials>,
        ) -> impl IntoResponse
            where
                <PermSet as PermissionSet>::Permission : Hash + Eq,
                User: axum_login::AuthUser<Id = String>,
        {
            let auth_res: Result<Option<User>, axum_login::Error<LoginFormAuthBackend<User,PswComparator,PermSet>>> =
                auth_session.authenticate(creds.clone()).await;
            let user = match auth_res {
                Ok(Some(user)) => user,
                Ok(None) => {
                    return LoginTemplate {
                            message: Some("Invalid credentials.".to_string()),
                            next: creds.next,
                        }
                        .into_response()
                }
                Err(err) => {
                    match err {
                        axum_login::Error::Session(err) => {
                            error!("Authentication session error [{}]", err)
                        }
                        axum_login::Error::Backend(err) => {
                            error!("Authentication backend error [{}]", err)
                        }
                    }
                    return StatusCode::INTERNAL_SERVER_ERROR.into_response()
                },
                // Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            };

            if auth_session.login(&user).await.is_err() {
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }

            if let Some(ref next) = creds.next {
                Redirect::to(next).into_response()
            } else {
                Redirect::to("/").into_response()
            }
        }

    }


    mod get {
        use core::fmt::Debug;
        use core::hash::Hash;
        use super::*;
        use crate::PasswordComparator;
        use crate::backend::{ LoginFormAuthBackend, psw_auth::PswUser };
        use crate::permission::PermissionSet;

        pub async fn login(Query(NextUrl { next }): Query<NextUrl>) -> LoginTemplate {
            LoginTemplate { message: None, next }
        }

        pub async fn logout <
            User: axum_login::AuthUser + PswUser,
            PswComparator: PasswordComparator + Debug + Clone + Send + Sync,
            // !!! We cannot use there default params (like EmptyPerm/AlwaysAllowedPermSet) because axum_login
            // uses type_id for looking data in the session.
            Perm: Hash + Eq + Debug + Clone + Send + Sync,
            PermSet: PermissionSet<Permission=Perm> + Clone,
        > (mut auth_session: axum_login::AuthSession<LoginFormAuthBackend<User,PswComparator,PermSet>>)
            -> impl IntoResponse
            where
                <PermSet as PermissionSet>::Permission : Hash + Eq,
                User: axum_login::AuthUser<Id = String>,
        {
            match auth_session.logout().await {
                Ok(_) => Redirect::to("/login").into_response(),
                Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            }
        }
    }

}
