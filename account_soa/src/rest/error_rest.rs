use axum::body::Body;
use axum::http::StatusCode;
use axum::Json;
use axum::response::{IntoResponse, Response };
use axum_extra::headers::Authorization;
use axum_extra::headers::authorization::Basic;
use axum_extra::TypedHeader;
use log::error;
use mvv_common::backtrace::{backtrace, BacktraceCell};
use mvv_common::entity::AmountFormatError;
use mvv_common::entity::currency::parse::CurrencyFormatError;
use mvv_common::entity::id::parse::IdFormatError;
use crate::entity::user::UserId;
use crate::service::account_service::AccountProcessError;
//--------------------------------------------------------------------------------------------------


// Error processing:
//  * https://docs.rs/axum/latest/axum/error_handling/
//  * https://github.com/tokio-rs/axum/blob/main/examples/anyhow-error-response/src/main.rs


// Make our own error that wraps `anyhow::Error`.
// #[derive(thiserror::Error)]
#[derive(
    Debug,
    thiserror::Error,
    mvv_error_macro::ThisErrorFromWithBacktrace,
    mvv_error_macro::ThisErrorBacktraceSource,
)]
pub enum RestAppError {
    #[error("AnyhowError({0})")]
    AnyhowError(#[source] anyhow::Error),

    // In most cases authentication should be processed on axum route layer.
    //
    // #[deprecated(note = "mainly for internal/automatic usage in macro when container is cloned.")]
    #[allow(unused_attributes)]
    #[must_use = "Mainly for xxx usage."]
    #[error("Unauthenticated")]
    Unauthenticated(UserId, BacktraceCell),

    // In most cases authorization should be processed on axum route layer,
    // but of course in some cases it is needed to do it later (or some additional checks)
    // (for example if user sends account ID of another client)
    #[error("Unauthorized")]
    Unauthorized(UserId, BacktraceCell),

    #[error("HttpResponseResultError")]
    #[no_source_backtrace]
    HttpResponseResultError(Response, BacktraceCell),

    #[error("IllegalArgument {0}")]
    IllegalArgument(#[source] anyhow::Error),
    #[error("ValidationRequestError {0}")]
    ValidationRequestError(#[source] Box<dyn std::error::Error>, BacktraceCell),

    #[error("ValidationError {0}")]
    ValidifyError(#[source] #[from_with_bt] validify::ValidationError, BacktraceCell),
    #[error("ValidationErrors {0}")]
    ValidifyErrors(#[source] #[from_with_bt] validify::ValidationErrors, BacktraceCell),

    #[error("AccountProcessError {0}")]
    AccountProcessError(#[source] #[from_with_bt] AccountProcessError, BacktraceCell),
    // ...
    // Add other errors if it is needed.
}

/*
impl fmt::Display for RestAppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RestAppError::AnyhowError(ref anyhow_err) =>
                write!(f, "AnyhowError: {}", anyhow_err),
            RestAppError::Unauthorized(..) =>
                write!(f, "NotAuthorized"),
            RestAppError::Unauthenticated(..) =>
                write!(f, "NotAuthenticated"),
            RestAppError::IllegalArgument(ref anyhow_err) =>
                write!(f, "IllegalArgument: {}", anyhow_err),
            RestAppError::HttpResponseResultError(ref _r) =>
                write!(f, "HttpResponseResultError"),
            RestAppError::ValidifyError(ref err, ..) =>
                write!(f, "ValidationError({err:?})"),
            RestAppError::ValidifyErrors(ref err, ..) =>
                write!(f, "ValidationErrors({err:?})"),
            RestAppError::ValidationRequestError(ref err, ..) =>
                write!(f, "ValidationRequestError({err:?})"),
        }
    }
}
*/


// Tell axum how to convert `AppError` into a response.
impl IntoResponse for RestAppError {
    fn into_response(self) -> Response {
        match self {
            RestAppError::AnyhowError(ref err) => {
                error!("Internal error: {err:?}");
                (StatusCode::INTERNAL_SERVER_ERROR, format!("Internal error: {}", err)).into_response()
            }
            RestAppError::Unauthenticated(ref user_id, ref backtrace) => {
                error!("Unauthenticated error ({user_id}) \n{backtrace}");
                StatusCode::UNAUTHORIZED.into_response()
            }
            RestAppError::Unauthorized(ref user_id, ref backtrace) => {
                error!("Unauthorized error ({user_id}) \n{backtrace}");
                (StatusCode::FORBIDDEN, "Unauthorized").into_response()
            }
            RestAppError::IllegalArgument(ref anyhow_err) => {
                error!("IllegalArgument error: {anyhow_err:?}");
                (StatusCode::BAD_REQUEST, format!("Illegal arguments: {}", anyhow_err)).into_response()
            }
            RestAppError::ValidifyError(ref err, ref backtrace) => {
                error!("ValidifyError: {err:?} \n {backtrace}");
                (StatusCode::BAD_REQUEST, Json(err.to_string())).into_response()
            }
            RestAppError::ValidifyErrors(ref err, ref backtrace) => {
                error!("ValidifyErrors: {err:?} \n {backtrace}");
                (StatusCode::BAD_REQUEST, Json(err.to_string())).into_response()
            }
            RestAppError::HttpResponseResultError(response, ref backtrace) => {
                error!("HttpResponseResultError: {response:?} \n {backtrace}");
                response
            },
            RestAppError::ValidationRequestError(ref err, ref backtrace) => {
                error!("ValidationRequestError: {err:?} \n {backtrace}");
                (StatusCode::BAD_REQUEST, Json(err.to_string())).into_response()
            }
            RestAppError::AccountProcessError(ref err, ref backtrace) => {
                error!("AccountProcessError: {err:?} \n {backtrace}");
                (StatusCode::BAD_REQUEST, Json(err.to_string())).into_response()
            }
        }
    }
}


impl From<uuid::Error> for RestAppError {
    fn from(err: uuid::Error) -> Self {
        RestAppError::ValidationRequestError(Box::new(err), backtrace())
    }
}
impl From<iban::ParseError> for RestAppError {
    fn from(err: iban::ParseError) -> Self {
        RestAppError::ValidationRequestError(Box::new(err), backtrace())
    }
}
impl From<IdFormatError> for RestAppError {
    fn from(err: IdFormatError) -> Self {
        let bt = BacktraceCell::inherit_or_capture(&err);
        RestAppError::ValidationRequestError(Box::new(err), bt)
    }
}
impl From<CurrencyFormatError> for RestAppError {
    fn from(err: CurrencyFormatError) -> Self {
        let bt = BacktraceCell::inherit_or_capture(&err);
        RestAppError::ValidationRequestError(Box::new(err), bt)
    }
}
impl From<AmountFormatError> for RestAppError {
    fn from(err: AmountFormatError) -> Self {
        let bt = BacktraceCell::inherit_or_capture(&err);
        RestAppError::ValidationRequestError(Box::new(err), bt)
    }
}


/*
// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, AppError>`. That way you don't need to do that manually.
impl<E> From<E> for RestAppError where E: Into<anyhow::Error> { // T O D O: remove it
    fn from(err: E) -> Self {
        RestAppError::AnyhowError(err.into())
    }
}
*/


pub fn test_authenticate_basic(creds: &Option<TypedHeader<Authorization<Basic>>>) -> Result<(), RestAppError> {
    let err_response: Response = Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .header("WWW-Authenticate", "Basic")
            .body(Body::from("Unauthenticated")) // or Body::empty() // Json(json!({"error": "Unauthorized"}))
            .unwrap_or_else(|_err| StatusCode::UNAUTHORIZED.into_response());

    match creds {
        None => return Err(RestAppError::HttpResponseResultError(err_response, backtrace())),
        Some(TypedHeader(Authorization(ref creds))) => {
            let usr = creds.username();
            let psw = creds.password();
            if usr != "test-rest-vovan" || psw != "qwerty" {
                return Err(RestAppError::HttpResponseResultError(err_response, backtrace()));
            }
        }
    }
    return Ok(());
}


/*

use axum::Json;
use serde_json::Value;


const SECRET_SIGNING_KEY: &[u8] = b"keep_th1s_@_secret";
#[derive(serde::Serialize, serde::Deserialize)]
pub struct OurJwtPayload {
    pub sub: String,
    pub exp: usize,
}
impl OurJwtPayload {
    pub fn new(sub: String) -> Self {
        use std::time::{ Duration, SystemTime };

        // expires by default in 60 minutes from now
        let exp = SystemTime::now()
            .checked_add(Duration::from_secs(60 * 60))
            .expect("valid timestamp")
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("valid duration")
            .as_secs() as usize;

        OurJwtPayload { sub, exp }
    }
}

pub fn verify_jwt(creds: &Basic) -> Result<(), (StatusCode, Json<Value>)> {
    use serde_json::json;
    use axum::Json;

    if let Ok(_jwt) = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &OurJwtPayload::new(creds.username().to_string()),
        &jsonwebtoken::EncodingKey::from_secret(SECRET_SIGNING_KEY),
    ) {
        // some validation...
        Ok(())
    } else {
        Err((
            StatusCode::UNAUTHORIZED, // // StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Failed to generate token"})),
        ))
    }
}
*/
