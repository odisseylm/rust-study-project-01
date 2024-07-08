use core::fmt;
use axum::body::Body;
use axum::http::StatusCode;
use axum::response::{ IntoResponse, Response };
use axum_extra::headers::Authorization;
use axum_extra::headers::authorization::Basic;
use axum_extra::TypedHeader;
//--------------------------------------------------------------------------------------------------


// Error processing:
//  * https://docs.rs/axum/latest/axum/error_handling/
//  * https://github.com/tokio-rs/axum/blob/main/examples/anyhow-error-response/src/main.rs


// Make our own error that wraps `anyhow::Error`.
// #[derive(thiserror::Error)]
#[derive(Debug)]
pub enum RestAppError {
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
    // ...
    // Add other errors if it is needed.
}

impl fmt::Display for RestAppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RestAppError::AnyhowError(ref anyhow_err) =>
                write!(f, "AnyhowError: {}", anyhow_err),
            RestAppError::Unauthorized =>
                write!(f, "NotAuthorized"),
            RestAppError::Unauthenticated =>
                write!(f, "NotAuthenticated"),
            RestAppError::IllegalArgument(ref anyhow_err) =>
                write!(f, "IllegalArgument: {}", anyhow_err),
            RestAppError::HttpResponseResultError(ref _r) =>
                write!(f, "HttpResponseResultError"),
        }
    }
}


// Tell axum how to convert `AppError` into a response.
impl IntoResponse for RestAppError {
    fn into_response(self) -> Response {
        match self {
            RestAppError::AnyhowError(ref err) =>
                ( StatusCode::INTERNAL_SERVER_ERROR, format!("Internal error: {}", err) ).into_response(),
            RestAppError::Unauthenticated =>
                StatusCode::UNAUTHORIZED.into_response(),
            RestAppError::Unauthorized =>
                ( StatusCode::FORBIDDEN, "Unauthorized" ).into_response(),
            RestAppError::IllegalArgument(ref err) =>
                ( StatusCode::BAD_REQUEST, format!("Illegal arguments: {}", err) ).into_response(),
            RestAppError::HttpResponseResultError(response) => response,
        }
    }
}


// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, AppError>`. That way you don't need to do that manually.
impl<E> From<E> for RestAppError where E: Into<anyhow::Error> {
    fn from(err: E) -> Self {
        RestAppError::AnyhowError(err.into())
    }
}


pub fn test_authenticate_basic(creds: &Option<TypedHeader<Authorization<Basic>>>) -> Result<(), RestAppError> {
    let err_response: Response = Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .header("WWW-Authenticate", "Basic")
            .body(Body::from("Unauthenticated")) // or Body::empty() // Json(json!({"error": "Unauthorized"}))
            .unwrap_or_else(|_err| StatusCode::UNAUTHORIZED.into_response());

    match creds {
        None => return Err(RestAppError::HttpResponseResultError(err_response)),
        Some(TypedHeader(Authorization(ref creds))) => {
            let usr = creds.username();
            let psw = creds.password();
            if usr != "test-rest-vovan" || psw != "qwerty" {
                return Err(RestAppError::HttpResponseResultError(err_response));
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
