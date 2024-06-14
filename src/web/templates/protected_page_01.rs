use axum::{ Router, routing::get as GET };


#[derive(askama::Template)]
#[template(path = "protected-01.html")]
struct ProtectedTemplate<'a> {
    username: &'a str,
}

pub fn protected_page_01_router() -> Router<()> {
    Router::new()
        .route("/protected-01", GET(get::protected_page_01))
        .route_layer(axum_login::login_required!(crate::rest::auth::AuthnBackend, login_url = "/login"))
}

mod get {
    use super::*;
    use crate::rest::auth::AuthSession;
    use axum::{ http::StatusCode, response::IntoResponse };

    pub async fn protected_page_01(auth_session: AuthSession) -> impl IntoResponse {
        match auth_session.user {
            Some(user) => ProtectedTemplate {
                username: &user.username,
            }.into_response(),

            None => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }
}
