use axum::body::Body;
use axum::http::StatusCode;
use axum::response::{ IntoResponse, Response };


pub(crate) fn http_unauthenticated_401_response(www_authenticate: &'static str) -> Response<Body> {
    Response::builder()
        .status(StatusCode::UNAUTHORIZED)
        // Other auth supported schemas: Bearer, Digest, OAuth, PrivateToken, so on.
        //   See https://www.iana.org/assignments/http-authschemes/http-authschemes.xhtml
        // .header("WWW-Authenticate", "Basic")
        .header("WWW-Authenticate", www_authenticate)
        .body(Body::from("Unauthenticated")) // or Body::empty() // Json(json!({"error": "Unauthorized"}))
        .unwrap_or_else(|_err| StatusCode::UNAUTHORIZED.into_response())
}

pub(crate) fn url_encode(string: &str) -> String {
    url::form_urlencoded::Serializer::new(String::new())
        .append_key_only(string)
        // .append_pair("foo", "bar & baz")
        // .append_pair("saison", "Été+hiver")
        .finish()
}
