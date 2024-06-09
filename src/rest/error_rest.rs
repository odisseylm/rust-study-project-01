
use core::fmt;
use axum::http::StatusCode;
use axum::response::IntoResponse;


// Error processing:
//  * https://docs.rs/axum/latest/axum/error_handling/
//  * https://github.com/tokio-rs/axum/blob/main/examples/anyhow-error-response/src/main.rs


// Make our own error that wraps `anyhow::Error`.
// #[derive(thiserror::Error)]
#[derive(Debug)]
pub enum AppRestError {
    AnyhowError(anyhow::Error),
    // ...
    // other errors if it is needed
}

impl fmt::Display for AppRestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppRestError::AnyhowError(ref anyhow_err) => { write!(f, "AnyhowError: {}", anyhow_err) }
        }
    }
}


// Tell axum how to convert `AppError` into a response.
impl IntoResponse for AppRestError {
    fn into_response(self) -> axum::response::Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self),
        ).into_response()
    }
}


// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, AppError>`. That way you don't need to do that manually.
impl<E> From<E> for AppRestError where E: Into<anyhow::Error> {
    fn from(err: E) -> Self {
        AppRestError::AnyhowError(err.into())
    }
}
