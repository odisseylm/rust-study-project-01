use core::fmt::{ self, Debug };
// use axum::Json;
use axum::response::{ IntoResponse, Response };
// use http::StatusCode;
use log::error;
use mvv_auth::UserId;
use mvv_common::string::{ SringOps, StaticRefOrString };
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
    Unauthorized(UserId),

    // #[error("HttpResponseResultError")]
    HttpResponseResultError(Response),

    // #[error("IllegalArgument({0})")]
    IllegalArgument(anyhow::Error),

    // #[error("IllegalArgument({0})")]
    ValidifyError(validify::ValidationError),
    ValidifyErrors(validify::ValidationErrors),

    RestCallError(mvv_common::soa::RestCallError)
    // ...
    // Add other errors if it is needed.
}

impl fmt::Display for WebAppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WebAppError::AnyhowError(ref anyhow_err) =>
                write!(f, "AnyhowError: {}", anyhow_err),
            WebAppError::Unauthorized(ref user_id) =>
                write!(f, "NotAuthorized user [{user_id}]"),
            WebAppError::Unauthenticated =>
                write!(f, "NotAuthenticated"),
            WebAppError::RestCallError(ref err) =>
                write!(f, "RestCallError: {}", err),
            WebAppError::IllegalArgument(ref anyhow_err) =>
                write!(f, "IllegalArgument: {}", anyhow_err),
            WebAppError::ValidifyError(ref err) =>
                write!(f, "ValidationError({err:?})"),
            WebAppError::ValidifyErrors(ref err) =>
                write!(f, "ValidationErrors({err:?})"),
            WebAppError::HttpResponseResultError(ref _r) =>
                write!(f, "HttpResponseResultError"),
        }
    }
}


// Tell axum how to convert `AppError` into a response.
impl IntoResponse for WebAppError {
    fn into_response(self) -> Response {
        match self {
            WebAppError::AnyhowError(ref err) => {
                error!("Internal error: {err:?}");
                // (StatusCode::INTERNAL_SERVER_ERROR, format!("Internal error: {}", err)).into_response()
            }
            WebAppError::Unauthenticated => {
                error!("Unauthenticated error");
                // StatusCode::UNAUTHORIZED.into_response()
            }
            WebAppError::Unauthorized(ref user_id) => {
                error!("Unauthorized access error (user: {user_id})");
                // (StatusCode::FORBIDDEN, "Unauthorized").into_response()
            }
            WebAppError::RestCallError(ref err) => {
                error!("RestCallError: {err:?}");
                // (StatusCode::FORBIDDEN, "Unauthorized").into_response()
            }
            WebAppError::IllegalArgument(ref err) => {
                error!("IllegalArgument error: {err:?}");
                // (StatusCode::BAD_REQUEST, format!("Illegal arguments: {}", err)).into_response()
            }
            WebAppError::ValidifyError(ref err) => {
                error!("ValidifyError error: {err:?}");
                // (StatusCode::BAD_REQUEST, Json(err.to_string())).into_response()
            }
            WebAppError::ValidifyErrors(ref err) => {
                error!("ValidifyErrors error: {err:?}");
                // (StatusCode::BAD_REQUEST, Json(err.to_string())).into_response()
            }
            WebAppError::HttpResponseResultError(response) => {
                error!("HttpResponseResultError error: {response:?}");
                return response
                // error_page(self.into_error_details()).into_response()
            },
        };

        error_page(self.into_error_details()).into_response()
    }
}


/*
// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, AppError>`. That way you don't need to do that manually.
impl<E> From<E> for WebAppError where E: Into<anyhow::Error> {
    fn from(err: E) -> Self {
        WebAppError::AnyhowError(err.into())
    }
}
*/

impl From<validify::ValidationErrors> for WebAppError {
    fn from(err: validify::ValidationErrors) -> Self {
        WebAppError::ValidifyErrors(err)
    }
}
impl From<validify::ValidationError> for WebAppError {
    fn from(err: validify::ValidationError) -> Self {
        WebAppError::ValidifyError(err)
    }
}
impl From<mvv_common::soa::RestCallError> for WebAppError {
    fn from(err: mvv_common::soa::RestCallError) -> Self {
        WebAppError::RestCallError(err.into())
    }
}

impl From<WebAppError> for ErrorDetails {
    fn from(err: WebAppError) -> Self {
        // TODO: simplify
        match err {
            WebAppError::AnyhowError(ref err) => {
                ErrorDetails {
                    title: "Internal error".into(),
                    short_description: err.to_string().into(),
                    full_description: Some(StaticRefOrString::String(
                        err.to_debug_err_string())),
                }
            }
            WebAppError::Unauthenticated => {
                ErrorDetails {
                    title: "Unauthenticated access".into(),
                    short_description: "Unauthenticated access".into(),
                    full_description: None,
                }
            }
            WebAppError::Unauthorized(ref user_id) => {
                ErrorDetails {
                    title: "Unauthorized access".into(),
                    short_description: "Unauthorized access".into(),
                    full_description: Some(format!("Unauthorized access for user [{user_id}]").into()),
                }
            }
            WebAppError::RestCallError(ref err) => {
                ErrorDetails {
                    title: "RestCallError".into(),
                    short_description: err.to_string().into(),
                    full_description: Some(StaticRefOrString::String(
                        err.to_debug_err_string())),
                }
            }
            WebAppError::IllegalArgument(ref err) => {
                ErrorDetails {
                    title: "IllegalArgument".into(),
                    short_description: err.to_string().into(),
                    full_description: Some(StaticRefOrString::String(
                        err.to_debug_err_string())),
                }
            }
            WebAppError::ValidifyError(ref err) => {
                ErrorDetails {
                    title: "Validation Error".into(),
                    short_description: err.to_string().into(),
                    full_description: Some(StaticRefOrString::String(
                        err.to_debug_err_string())),
                }
            }
            WebAppError::ValidifyErrors(ref err) => {
                ErrorDetails {
                    title: "Validation Error".into(),
                    short_description: err.to_string().into(),
                    full_description: Some(StaticRefOrString::String(
                        err.to_debug_err_string())),
                }
            }
            WebAppError::HttpResponseResultError(_) => {
                ErrorDetails {
                    title: "ResponseResultError".into(),
                    short_description: err.to_string().into(),
                    full_description: Some(StaticRefOrString::String(
                        err.to_debug_err_string())),
                }
            }
        }
    }
}


#[extension_trait::extension_trait]
pub impl<T> ErrDebugStrExt for T /* where T: Debug */ {
    #[track_caller]
    fn to_debug_err_string(&self) -> String where Self: Debug {
        self.to_debug_string()
        // add other possible chars filtering if it is needed
    }
}


pub struct ErrorDetails {
    pub title: &'static str,
    pub short_description: StaticRefOrString,
    pub full_description: Option<StaticRefOrString>,
}

#[extension_trait::extension_trait]
pub impl<Err> IntoErrorDetailsExt for Err where Err: Into<WebAppError> {
    fn into_error_details(self) -> ErrorDetails {
        let err: WebAppError = self.into();
        err.into()
    }
}


#[derive(askama::Template)]
#[template(path = "error_page.html")]
struct ErrorPageTemplate<'a> {
    error: &'a ErrorDetails,
}


// pub fn error_page_router() -> axum::Router<()> {
//     use axum::{ Router, routing::get as GET };
//     Router::new().route("/error", GET(error_page))
// }


pub fn error_page(error_details: ErrorDetails) -> impl IntoResponse {
    ErrorPageTemplate {
        error: &error_details,
    }.into_response()
}
