
use core::fmt;
use axum::body::Body;
use axum::http::response::Parts;
use axum::http::StatusCode;
use axum::response::IntoResponse;


// Error processing:
//  * https://docs.rs/axum/latest/axum/error_handling/
//  * https://github.com/tokio-rs/axum/blob/main/examples/anyhow-error-response/src/main.rs


// Make our own error that wraps `anyhow::Error`.
// #[derive(thiserror::Error)]
#[derive(Debug)]
pub enum RestAppError {
    AnyhowError(anyhow::Error),
    Unauthenticated,
    Unauthorized,
    IllegalArgument(anyhow::Error),
    // ...
    // other errors if it is needed
}

impl fmt::Display for RestAppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RestAppError::AnyhowError(ref anyhow_err) => { write!(f, "AnyhowError: {}", anyhow_err) }
            RestAppError::Unauthorized => { write!(f, "NotAuthorized") }
            RestAppError::Unauthenticated => { write!(f, "NotAuthenticated") }
            RestAppError::IllegalArgument(ref anyhow_err) => { write!(f, "AnyhowError: {}", anyhow_err) }
        }
    }
}


// Tell axum how to convert `AppError` into a response.
impl IntoResponse for RestAppError {
    fn into_response(self) -> axum::response::Response {
        match self {
            RestAppError::AnyhowError(ref err) =>
                ( StatusCode::INTERNAL_SERVER_ERROR, format!("Internal error: {}", err) ).into_response(),
            RestAppError::Unauthenticated => {
                axum::response::Response::builder()
                    .status(StatusCode::UNAUTHORIZED)
                    .header("WWW-Authenticate", "Basic")
                    .body(Body::from("Unauthenticated")) // or Body::empty()
                    .unwrap_or_else(|_err| StatusCode::UNAUTHORIZED.into_response())
            }
            RestAppError::Unauthorized =>
                ( StatusCode::FORBIDDEN, "Unauthorized".to_string() ).into_response(),
            RestAppError::IllegalArgument(ref err) =>
                ( StatusCode::BAD_REQUEST, format!("Illegal arguments: {}", err) ).into_response(),
        }//.into_response()
    }
}


// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, AppError>`. That way you don't need to do that manually.
impl<E> From<E> for RestAppError where E: Into<anyhow::Error> {
    fn from(err: E) -> Self {
        RestAppError::AnyhowError(err.into())
    }
}
