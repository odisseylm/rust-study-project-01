use std::sync::Arc;
use async_trait::async_trait;
//--------------------------------------------------------------------------------------------------


use crate::user_provider::AuthUserProvider;


// As custom grpc/http request extension
#[derive(Debug, Clone)]
pub struct UserContext<Usr: axum_login::AuthUser> {
    user: Usr,
    // probably later we can add there Option<RoleSet>
}
impl <Usr: axum_login::AuthUser> UserContext<Usr> {
    pub fn new(user: Usr) -> Self {
        Self { user }
    }
    pub fn user(&self) -> &Usr {
        &self.user
    }
}


#[derive(Debug, Copy, Clone)]
pub enum AuthBackendMode {
    AuthSupported,
    AuthProposed,
}


/// In case of
/// * HTTP Basic authentication it is sending corresponding HTTP header
/// * Login form authentication it is redirecting to HTML form
/// * OAuth also redirecting to form page
///
#[allow(dead_code)] // It is used as type marker (we could directly use IntoResponse, but...)
pub trait ProposeAuthAction : axum::response::IntoResponse {
}

#[derive(Debug, Clone)]
pub struct NoProposeHttpAuthAction;
impl ProposeAuthAction for NoProposeHttpAuthAction { }
#[inherent::inherent]
impl axum::response::IntoResponse for NoProposeHttpAuthAction {
    #[allow(dead_code)] // !! It is really used IMPLICITLY !!
    pub fn into_response(self) -> axum::response::Response<axum::body::Body> {
        // Should not be called
        http_unauthenticated_401_response("Unknown")
    }
}

#[async_trait]
#[cfg_attr(feature = "ambassador", ambassador::delegatable_trait)]
pub trait AuthnBackendAttributes : axum_login::AuthnBackend + Clone + Send + Sync {
    type ProposeAuthAction: ProposeAuthAction;
    fn user_provider(&self) -> Arc<dyn AuthUserProvider<User = Self::User> + Sync + Send>;
    fn user_provider_ref<'a>(&'a self) -> &'a Arc<dyn AuthUserProvider<User = Self::User> + Sync + Send>;
    fn propose_authentication_action(&self, req: &axum::extract::Request) -> Option<Self::ProposeAuthAction>;
}


#[async_trait]
#[cfg_attr(feature = "ambassador", ambassador::delegatable_trait)]
pub trait RequestAuthenticated : axum_login::AuthnBackend + Clone + Send + Sync {

    /// Authenticates the request credentials with the backend if it is present in request
    /// or use user from HTTP session.
    ///
    /// Since we cannot pass Request (because we cannot clone it) and cannot pass &Request due to
    /// future compilation error, we will extract required request data and pass it to authenticate_request.
    //
    // Workaround with returning moved request. Probably not good approach... Hz...
    //
    async fn do_authenticate_request <
        RootBackend: axum_login::AuthnBackend + 'static,
        S: Send + Sync,
    > (&self, auth_session: Option<axum_login::AuthSession<RootBackend>>, req: axum::extract::Request)
        -> (axum::extract::Request, Result<Option<Self::User>, Self::Error>)
        where Self: 'static,
              RootBackend: axum_login::AuthnBackend<User = Self::User>,
    {
        if let Some(auth_session) = auth_session {
            self.internal_do_authenticate_request_by_user_session
                ::<RootBackend, S>(auth_session, req).await
        } else {
            (req, Ok(None))
        }
    }

    async fn internal_do_authenticate_request_by_user_session <
        RootBackend: axum_login::AuthnBackend<User=Self::User> + 'static,
        S: Send + Sync,
    >
    (&self, auth_session: axum_login::AuthSession<RootBackend>, req: axum::extract::Request)
        -> (axum::extract::Request, Result<Option<Self::User>, Self::Error>)
        where Self: 'static {

        // It works Ok.
        // let auth_session22: Option<axum_login::AuthSession<RootBackend>> = req.extensions()
        //     .get::<axum_login::AuthSession<RootBackend>>().cloned();

        match auth_session.user {
            None =>
                (req, Ok(None)),
            Some(auth_session_user) =>
                (req, Ok(Some(auth_session_user))),
        }
    }

    /// Authenticates the request credentials with the backend if it is present in request
    /// or use user from HTTP session.
    ///
    /// Since we cannot pass Request (because we cannot clone it) and cannot pass &Request due to
    /// future compilation error, we will extract required request data and pass it to authenticate_request.
    ///
    async fn do_authenticate_request_parts <
        RootBackend: axum_login::AuthnBackend + 'static,
        S: Send + Sync,
    > (&self, auth_session: Option<axum_login::AuthSession<RootBackend>>, _req: &http::request::Parts)
        -> Result<Option<Self::User>, Self::Error>
        where Self: 'static,
              RootBackend: axum_login::AuthnBackend<User = Self::User>,
    {
        if let Some(auth_session) = auth_session {
            self.internal_do_authenticate_request_parts_by_user_session
                ::<RootBackend, S>(auth_session).await
        } else {
            Ok(None)
        }
    }

    async fn internal_do_authenticate_request_parts_by_user_session <
        RootBackend: axum_login::AuthnBackend<User=Self::User> + 'static,
        S: Send + Sync,
    >
    (&self, auth_session: axum_login::AuthSession<RootBackend>)
        -> Result<Option<Self::User>, Self::Error>
        where Self: 'static {

        // It works Ok.
        // let auth_session22: Option<axum_login::AuthSession<RootBackend>> = req.extensions()
        //     .get::<axum_login::AuthSession<RootBackend>>().cloned();

        match auth_session.user {
            None =>
                Ok(None),
            Some(auth_session_user) =>
                Ok(Some(auth_session_user)),
        }
    }
}


mod axum_login_delegatable;
pub mod http_basic_auth;
pub mod login_form_auth;
pub mod oauth2_auth;
pub mod psw_auth;
pub mod authz_backend;
mod client_cert_auth;

pub use http_basic_auth::{HttpBasicAuthBackend, ProposeHttpBasicAuthAction, };
pub use login_form_auth::{ LoginFormAuthBackend, LoginFormAuthConfig, ProposeLoginFormAuthAction, };
pub use oauth2_auth::{ OAuth2AuthBackend, OAuth2AuthCredentials, OAuth2Config, Oauth2ConfigError, OAuth2UserStore, };
pub use client_cert_auth::{ClientCertAuthBackend, ClientCertAuthCredentials};
pub use psw_auth::{ PswAuthCredentials };
use crate::util::http::http_unauthenticated_401_response;
