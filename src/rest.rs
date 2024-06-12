
pub mod account_rest;
pub mod dto;
pub mod error_rest;
mod rest_investigation;
pub mod app_dependencies;
pub mod utils;
pub mod web_app;

// mod rest_auth;
// pub mod auth {
//     // pub use super::AuthUser;
//     pub use crate::rest::rest_auth::validate_auth;
//     pub use crate::rest::rest_auth::AuthUser;
//     pub use crate::rest::rest_auth::AuthnBackend;
//     pub use crate::rest::rest_auth::AuthSession;
// }
pub mod auth;

pub mod axum_login_investigation;
pub mod oauth;
