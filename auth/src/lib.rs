
// pub mod auth;

mod error;

mod psw;

pub mod util;
pub mod backend;
pub mod user_provider;

pub mod examples;
pub mod permission;
mod test;
pub mod route;
pub mod user_id;
pub mod extract;
mod secure_str;

pub mod psw_hash;

pub use user_id::UserId;

pub use backend::{ AuthBackendMode, AuthnBackendAttributes, ProposeAuthAction };
pub use error::AuthBackendError;

pub use user_provider::{ AuthUserProvider, AuthUserProviderError };

pub use psw::{ PasswordComparator, PlainPasswordComparator };
pub use secure_str::{ SecureString, clear_string_chars };

pub mod http {
    use axum::extract::OriginalUri;

    pub fn req_original_uri(req: &axum::extract::Request) -> Option<String> {
        let url: Option<String> = req.extensions().get::<OriginalUri>()
            .map(|uri|uri.to_string());
        url
    }

    pub fn req_original_uri_or_empty(req: &axum::extract::Request) -> String {
        req_original_uri(req).unwrap_or_else(||String::new())
    }

}