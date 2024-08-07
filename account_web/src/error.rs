use core::fmt::{ self, Debug };
// use axum::Json;
use axum::response::{ IntoResponse, Response };
// use http::StatusCode;
use log::error;
use mvv_auth::UserId;
use mvv_common::backtrace::{ backtrace, BacktraceCell };
use mvv_common::string::{ SringOps, StaticRefOrString };
//--------------------------------------------------------------------------------------------------



#[derive(
    Debug,
    thiserror::Error,
    mvv_error_macro::ThisErrorFromWithBacktrace,
    mvv_error_macro::ThisErrorBacktraceSource,
)]
pub enum WebAppError {
    #[error("AnyhowError({0})")]
    AnyhowError(anyhow::Error),

    // Ideally it should not be used in normal app flow.
    // Authentication should be performed on axum route layer.
    //
    // #[deprecated(note = "mainly for internal/automatic usage in macro when container is cloned.")]
    #[allow(unused_attributes)]
    #[must_use = "Mainly for xxx usage."]
    #[error("Unauthenticated")]
    Unauthenticated(UserId, BacktraceCell),

    // In most cases authorization also should be processed on axum route layer,
    // but of course in some cases it is possible to do only later
    // (for example if user sends account ID of another client)
    #[error("Unauthorized")]
    Unauthorized(UserId, BacktraceCell),

    #[error("HttpResponseResultError")]
    HttpResponseResultError(Response, BacktraceCell),

    #[error("IllegalArgument({0})")]
    IllegalArgument(anyhow::Error),

    #[error("ValidifyError({0})")]
    ValidifyError(#[source] #[from_with_bt] validify::ValidationError, BacktraceCell),
    #[error("ValidifyErrors({0})")]
    ValidifyErrors(#[source] #[from_with_bt] validify::ValidationErrors, BacktraceCell),

    #[error("RestCallError({0})")]
    RestCallError(#[from] mvv_common::soa::RestCallError)
    // ...
    // Add other errors if it is needed.
}



// Tell axum how to convert `AppError` into a response.
impl IntoResponse for WebAppError {
    fn into_response(self) -> Response {
        match self {
            WebAppError::AnyhowError(ref err) => {
                error!("Internal error: {err:?}");
            }
            WebAppError::Unauthenticated(ref user_id, ref backtrace) => {
                error!("Unauthenticated error (user: {user_id}) \n {backtrace}");
            }
            WebAppError::Unauthorized(ref user_id, ref backtrace) => {
                error!("Unauthorized access error (user: {user_id}) \n {backtrace}");
            }
            WebAppError::RestCallError(ref err) => {
                error!("RestCallError: {err:?}");
            }
            WebAppError::IllegalArgument(ref err) => {
                error!("IllegalArgument error: {err:?}");
            }
            WebAppError::ValidifyError(ref err, ref backtrace) => {
                error!("ValidifyError error: {err:?} \n {backtrace}");
            }
            WebAppError::ValidifyErrors(ref err, ref backtrace) => {
                error!("ValidifyErrors error: {err:?} \n {backtrace}");
            }
            WebAppError::HttpResponseResultError(response, ref backtrace) => {
                error!("HttpResponseResultError error: {response:?} \n {backtrace}");
                return response
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
            WebAppError::Unauthenticated(..) => {
                ErrorDetails {
                    title: "Unauthenticated access".into(),
                    short_description: "Unauthenticated access".into(),
                    full_description: None,
                }
            }
            WebAppError::Unauthorized(ref user_id, ..) => {
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
            WebAppError::ValidifyError(ref err, ..) => {
                ErrorDetails {
                    title: "Validation Error".into(),
                    short_description: err.to_string().into(),
                    full_description: Some(StaticRefOrString::String(
                        err.to_debug_err_string())),
                }
            }
            WebAppError::ValidifyErrors(ref err, ..) => {
                ErrorDetails {
                    title: "Validation Error".into(),
                    short_description: err.to_string().into(),
                    full_description: Some(StaticRefOrString::String(
                        err.to_debug_err_string())),
                }
            }
            WebAppError::HttpResponseResultError(..) => {
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
