
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


pub use auth_user::AuthUser;
pub use error::AuthBackendError;
pub use error::UnauthenticatedAction;

pub use auth_user_provider::AuthUserProvider;
pub use auth_user_provider::AuthUserProviderError;
pub use oauth2_auth::OAuth2UserStore;
pub use mem_user_provider::InMemAuthUserProvider;
pub use sql_user_provider::SqlUserProvider;

// pub use auth_user_provider::wrap_static_arc_auth_user_provider;
// pub use auth_user_provider_wrap::wrap_static_ptr_auth_user_provider;

pub use psw::PasswordComparator;
pub use psw::PlainPasswordComparator;

pub use http_basic_auth::HttpBasicAuthBackend;
// pub use http_basic_auth::HttpBasicAuthSession;
pub use http_basic_auth::HttpBasicAuthMode;
// pub use http_basic_auth::BasicAuthCreds;

pub use login_form_auth::LoginFormAuthBackend;
pub use login_form_auth::LoginFormAuthAuthSession;
pub use login_form_auth::LoginFormAuthMode;

pub use oauth2_auth::{ OAuth2AuthBackend, OAuth2AuthCredentials };
