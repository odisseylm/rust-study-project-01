
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

pub(crate) mod temp;
mod auth_user_provider_wrap;


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

pub use psw_auth::{ AuthBackend as PswAuthBackend, AuthCredentials as PswAuthCredentials, AuthSession as PswAuthSession };
pub use oauth2_auth::{ AuthBackend as OAuth2AuthBackend, AuthCredentials as OAuth2AuthCredentials, AuthSession as OAuth2AuthSession };
