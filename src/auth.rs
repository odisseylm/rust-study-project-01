
mod auth_user;

pub mod psw_auth;
pub mod oauth2_auth;
pub mod composite_auth;


mod auth_user_provider;
mod mem_user_provider;
mod sql_user_provider;


mod authn_backend_dyn_wrap;
mod error;

mod psw;

// pub(crate) mod temp;
mod auth_user_provider_wrap;
mod http;
mod auth_backend;
mod http_basic_auth;
mod login_form_auth;
mod auth_backend_delegate;
pub mod composite_util;


pub use auth_user::AuthUser;
pub use auth_backend::{ AuthBackendMode, AuthnBackendAttributes, RequestUserAuthnBackend, ProposeAuthAction };
pub use error::AuthBackendError;

pub use auth_user_provider::{ AuthUserProvider, AuthUserProviderError };
pub use http_basic_auth::{ HttpBasicAuthBackend, BasicAuthCreds, ProposeHttpBasicAuthAction };
pub use login_form_auth::{ LoginFormAuthBackend, LoginFormAuthConfig, ProposeLoginFormAuthAction };
pub use oauth2_auth::{ OAuth2AuthBackend, OAuth2UserStore, OAuth2AuthCredentials, OAuth2Config, Oauth2ConfigError };

pub use mem_user_provider::InMemAuthUserProvider;
pub use sql_user_provider::SqlUserProvider;

pub use psw::{ PasswordComparator, PlainPasswordComparator };
