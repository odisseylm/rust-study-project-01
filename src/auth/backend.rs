pub mod http_basic_auth;
pub mod login_form_auth;
pub mod oauth2_auth;
pub mod psw_auth;
pub mod axum_login_delegatable;

pub use http_basic_auth::{ BasicAuthCreds, HttpBasicAuthBackend, ProposeHttpBasicAuthAction };
pub use login_form_auth::{ LoginFormAuthBackend, LoginFormAuthConfig, ProposeLoginFormAuthAction };
pub use oauth2_auth::{ OAuth2AuthBackend, OAuth2AuthCredentials, OAuth2Config, Oauth2ConfigError, OAuth2UserStore };
// pub use composite_auth::{ CompositeAuthnBackend, CompositeAuthCredentials };
pub use psw_auth::{ PswAuthCredentials };
