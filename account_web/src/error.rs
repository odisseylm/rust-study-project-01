use core::fmt;
use axum::Json;
use axum::response::{IntoResponse, Response};
use http::StatusCode;
use log::error;
//--------------------------------------------------------------------------------------------------



#[derive(Debug)]
pub enum WebAppError {
    // #[error("AnyhowError({0})")]
    AnyhowError(/*#[from]*/ anyhow::Error),

    // Ideally it should not be used in normal app flow.
    // Authentication should be performed on axum route layer.
    //
    // #[deprecated(note = "mainly for internal/automatic usage in macro when container is cloned.")]
    #[allow(unused_attributes)]
    #[must_use = "Mainly for xxx usage."]
    // #[error("Unauthenticated")]
    Unauthenticated,

    // In most cases authorization also should be processed on axum route layer,
    // but of course in some cases it is possible to do only later
    // (for example if user sends account ID of another client)
    // #[error("Unauthorized")]
    Unauthorized,

    // #[error("HttpResponseResultError")]
    HttpResponseResultError(Response),

    // #[error("IllegalArgument({0})")]
    IllegalArgument(anyhow::Error),

    // #[error("IllegalArgument({0})")]
    ValidifyError(validify::ValidationError),
    ValidifyErrors(validify::ValidationErrors),
    // ...
    // Add other errors if it is needed.
}

impl fmt::Display for WebAppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WebAppError::AnyhowError(ref anyhow_err) =>
                write!(f, "AnyhowError: {}", anyhow_err),
            WebAppError::Unauthorized =>
                write!(f, "NotAuthorized"),
            WebAppError::Unauthenticated =>
                write!(f, "NotAuthenticated"),
            WebAppError::IllegalArgument(ref anyhow_err) =>
                write!(f, "IllegalArgument: {}", anyhow_err),
            WebAppError::HttpResponseResultError(ref _r) =>
                write!(f, "HttpResponseResultError"),
            WebAppError::ValidifyError(ref err) =>
                write!(f, "ValidationError({err:?})"),
            WebAppError::ValidifyErrors(ref err) =>
                write!(f, "ValidationErrors({err:?})"),
        }
    }
}


// Tell axum how to convert `AppError` into a response.
impl IntoResponse for WebAppError {
    fn into_response(self) -> Response {
        match self {
            WebAppError::AnyhowError(ref err) => {
                error!("Internal error: {err:?}");
                (StatusCode::INTERNAL_SERVER_ERROR, format!("Internal error: {}", err)).into_response()
            }
            WebAppError::Unauthenticated => {
                error!("Unauthenticated error");
                StatusCode::UNAUTHORIZED.into_response()
            }
            WebAppError::Unauthorized => {
                error!("Unauthorized error");
                (StatusCode::FORBIDDEN, "Unauthorized").into_response()
            }
            WebAppError::IllegalArgument(ref err) => {
                error!("IllegalArgument error: {err:?}");
                (StatusCode::BAD_REQUEST, format!("Illegal arguments: {}", err)).into_response()
            }
            WebAppError::ValidifyError(err) => {
                error!("ValidifyError error: {err:?}");
                (StatusCode::BAD_REQUEST, Json(err.to_string())).into_response()
            }
            WebAppError::ValidifyErrors(err) => {
                error!("ValidifyErrors error: {err:?}");
                (StatusCode::BAD_REQUEST, Json(err.to_string())).into_response()
            }
            WebAppError::HttpResponseResultError(response) => {
                error!("HttpResponseResultError error: {response:?}");
                response
            },
        }
        //
        //
        // match self {
        //     WebAppError::AnyhowError(ref err) =>
        //         ( StatusCode::INTERNAL_SERVER_ERROR, format!("Internal error: {}", err) ).into_response(),
        //     WebAppError::Unauthenticated =>
        //         StatusCode::UNAUTHORIZED.into_response(),
        //     WebAppError::Unauthorized =>
        //         ( StatusCode::FORBIDDEN, "Unauthorized" ).into_response(),
        //     WebAppError::IllegalArgument(ref err) =>
        //         ( StatusCode::BAD_REQUEST, format!("Illegal arguments: {}", err) ).into_response(),
        //     WebAppError::ValidifyError(err) =>
        //         ( StatusCode::BAD_REQUEST, Json(err.to_string()) ).into_response(),
        //     WebAppError::ValidifyErrors(err) =>
        //         ( StatusCode::BAD_REQUEST, Json(err.to_string()) ).into_response(),
        //     WebAppError::HttpResponseResultError(response) => response,
        // }
    }
}


// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, AppError>`. That way you don't need to do that manually.
impl<E> From<E> for WebAppError where E: Into<anyhow::Error> {
    fn from(err: E) -> Self {
        WebAppError::AnyhowError(err.into())
    }
}

/*
impl From<validify::ValidationErrors> for WebAppError {
    fn from(err: validify::ValidationErrors) -> Self {
        WebAppError::ValidifyErrors(err.into())
    }
}
impl From<validify::ValidationError> for WebAppError {
    fn from(err: validify::ValidationError) -> Self {
        WebAppError::ValidifyError(err.into())
    }
}
*/
