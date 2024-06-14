
use core::fmt;
use axum::body::Body;
use axum::http::StatusCode;
use axum::Json;
use axum::response::IntoResponse;
use axum_extra::headers::Authorization;
use axum_extra::headers::authorization::Basic;
use axum_extra::TypedHeader;
use serde_json::Value;
use crate::auth::UnauthenticatedAction;


// Error processing:
//  * https://docs.rs/axum/latest/axum/error_handling/
//  * https://github.com/tokio-rs/axum/blob/main/examples/anyhow-error-response/src/main.rs


// Make our own error that wraps `anyhow::Error`.
// #[derive(thiserror::Error)]
#[derive(Debug)]
pub enum RestAppError {
    AnyhowError(anyhow::Error),
    Unauthenticated(UnauthenticatedAction),
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
            RestAppError::Unauthenticated(_) => { write!(f, "NotAuthenticated") }
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
            RestAppError::Unauthenticated(propose_action) =>
                propose_action.into_response(),
            RestAppError::Unauthorized =>
                ( StatusCode::FORBIDDEN, "Unauthorized".to_string() ).into_response(),
            RestAppError::IllegalArgument(ref err) =>
                ( StatusCode::BAD_REQUEST, format!("Illegal arguments: {}", err) ).into_response(),
        }
    }
}

impl IntoResponse for UnauthenticatedAction {
    fn into_response(self) -> askama_axum::Response {
        match self {
            UnauthenticatedAction::NoAction =>
                StatusCode::UNAUTHORIZED.into_response(),

            UnauthenticatedAction::ProposeBase64 =>
                axum::response::Response::builder()
                    .status(StatusCode::UNAUTHORIZED)
                    .header("WWW-Authenticate", "Basic")
                    .body(Body::empty())
                    .unwrap_or_else(|_err| StatusCode::UNAUTHORIZED.into_response()),

            UnauthenticatedAction::ProposeLoginForm { login_form_url, initial_url } => {
                let login_url = login_form_url.unwrap_or("/login");
                let login_url = match initial_url {
                    None => login_url.to_string(),
                    Some(ref initial_url) => format!("{}?next={}", login_url, url_encode(initial_url.as_str())),
                };
                axum::response::Response::builder()
                    // .status(StatusCode::UNAUTHORIZED)
                    // .status(StatusCode::FOUND) // redirect
                    .status(StatusCode::UNAUTHORIZED) // redirect
                    .header("Location", login_url.clone())
                    .header("Content-Type", "text/html; charset=utf-8")
                    .body(Body::from(REDIRECT_LOGIN_PAGE_CONTENT.replace("{login_url}", login_url.as_str())))
                    .unwrap_or_else(|_err| StatusCode::UNAUTHORIZED.into_response())
            }
        }
    }
}

fn url_encode(string: &str) -> String {
    url::form_urlencoded::Serializer::new(String::new())
        .append_key_only(string)
        // .append_pair("foo", "bar & baz")
        // .append_pair("saison", "Été+hiver")
        .finish()
}

static REDIRECT_LOGIN_PAGE_CONTENT: &'static str = r#"
<!doctype html>
<html>
  <head>
    <meta charset="utf-8" />
    <meta http-equiv="refresh" content="0; url={login_url}">
    <title>User is not athenticated</title>
  </head>
  <body>
    <a href="{login_url}">Login</a>
  </body>
</html>
"#;

/*
// I do not want to add additional lib only for this html block.
#[static_init::dynamic]
static REDIRECT_LOGIN_PAGE_CONTENT_XML: inline_xml::Xml = inline_xml::xml!(
    // <!doctype html> // !!! It is not supported by inline_xml
    <html>
      <head>
        <meta charset="utf-8" />
        <meta http-equiv="refresh" content="300; url=/login?next={redirect}" />
        <title>User is not authenticated</title>
      </head>
      <body>
        <a href="/login?next={redirect}">Login</a>
      </body>
    </html>
);
*/
/*
fn aa() {
    let event: quick_xml::events::Event = xml_macro::xml!(<person name="Josh"
                         occupation={
                             let arr = ["a", "b", "c"];
                             arr[2]
                         }>);
}

// static REDIRECT_LOGIN_PAGE_CONTENT_XML2: &'static str = xml_macro::xml!(
static REDIRECT_LOGIN_PAGE_CONTENT_XML2: quick_xml::events::Event = xml_macro::xml!(
    <html>
      <head>
        <meta charset="utf-8" />
        <meta http-equiv="refresh" content="300; url=/login?redirect={redirect}" />
        <title>User is not athenticated</title>
      </head>
      <body>
        <a href="/login?redirect={redirect}">Login</a>
      </body>
    </html>
);
*/

/*
pub fn unauthenticated_401_response() -> Response<Body> {
    axum::response::Response::builder()
        .status(StatusCode::UNAUTHORIZED)
        // To show user dialog in web-browser; can be removed in prod.
        // Also, other auth schemas can be there (Bearer, Digest, OAuth, PrivateToken, etc.)
        //   See https://www.iana.org/assignments/http-authschemes/http-authschemes.xhtml
        .header("WWW-Authenticate", "Basic")
        .body(Body::from("Unauthenticated")) // or Body::empty() // Json(json!({"error": "Unauthorized"}))
        .unwrap_or_else(|_err| StatusCode::UNAUTHORIZED.into_response())
}
*/


// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, AppError>`. That way you don't need to do that manually.
impl<E> From<E> for RestAppError where E: Into<anyhow::Error> {
    fn from(err: E) -> Self {
        RestAppError::AnyhowError(err.into())
    }
}


pub fn authenticate_basic(creds: &Option<TypedHeader<Authorization<Basic>>>) -> Result<(), RestAppError> {
    match creds {
        None => return Err(RestAppError::Unauthenticated(UnauthenticatedAction::ProposeBase64)),
        Some(TypedHeader(Authorization(ref creds))) => {
            let usr = creds.username();
            let psw = creds.password();
            if usr != "vovan" || psw != "qwerty" {
                return Err(RestAppError::Unauthorized)
            }
        }
    }
    return Ok(());
}


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

// pub fn verify_jwt<C:Credentials>(creds: &C) {
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
