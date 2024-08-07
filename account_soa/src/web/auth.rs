use std::fmt::Display;
use askama::Template;
use axum::{
    extract::Query,
    http::StatusCode,
    response::{ IntoResponse, Redirect },
    routing::{ get as GET, post as POST },
    Form, Router,
};
use serde::Deserialize;
//--------------------------------------------------------------------------------------------------


pub const NEXT_URL_KEY: &str = "auth.next-url";

#[derive(Template)]
#[template(path = "login.html")]
pub struct LoginTemplate <Msg: Display> {
    pub message: Option<Msg>,
    pub next: Option<String>,
}

// This allows us to extract the "next" field from the query string. We use this
// to redirect after log in.
#[derive(Debug, Deserialize)]
pub struct NextUrl {
    next: Option<String>,
}

pub fn composite_login_router() -> Router<()> {
    Router::new()
        .route("/login/password", POST(post::login::password))
        .route("/login/oauth", POST(post::login::oauth))
        .route("/login", GET(get::login))
        .route("/logout", GET(get::logout))
}

mod post {
    use super::*;

    pub(super) mod login {
        use log::error;
        use crate::rest::auth::{ AuthUser, CompositeAuthBackend, CompositeAuthCredentials };
        use mvv_auth::backend::PswAuthCredentials as PasswordCreds;
        use super::*;

        pub async fn password(
            mut auth_session: axum_login::AuthSession<CompositeAuthBackend>,
            Form(creds): Form<PasswordCreds>,
        ) -> impl IntoResponse {
            let auth_res: Result<Option<AuthUser>, axum_login::Error<CompositeAuthBackend>> =
                auth_session.authenticate(
                    CompositeAuthCredentials::Password(creds.clone())).await;
            let user = match auth_res {
                Ok(Some(user)) => user,
                Ok(None) => {
                    return LoginTemplate {
                            message: Some("Invalid credentials."),
                            next: creds.next,
                        }
                        .into_response()
                }
                Err(err) => {
                    match err {
                        axum_login::Error::Session(err) => {
                            error!("Authentication session error [{}]", err)
                        }
                        axum_login::Error::Backend(err) => {
                            error!("Authentication backend error [{}]", err)
                        }
                    }
                    return StatusCode::INTERNAL_SERVER_ERROR.into_response()
                },
                // Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            };

            if auth_session.login(&user).await.is_err() {
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }

            if let Some(ref next) = creds.next {
                Redirect::to(next).into_response()
            } else {
                Redirect::to("/").into_response()
            }
        }

        pub async fn oauth(
            auth_session: axum_login::AuthSession<CompositeAuthBackend>,
            session: axum_login::tower_sessions::Session,
            Form(NextUrl { next }): Form<NextUrl>,
        ) -> impl IntoResponse {
            use crate::rest::auth::oauth::CSRF_STATE_KEY;

            let Ok((auth_url, csrf_state)) = auth_session.backend.authorize_url()
                else { return StatusCode::INTERNAL_SERVER_ERROR.into_response() };

            session
                .insert(CSRF_STATE_KEY, csrf_state.secret())
                .await
                .expect("Serialization should not fail.");

            session
                .insert(NEXT_URL_KEY, next)
                .await
                .expect("Serialization should not fail.");

            Redirect::to(auth_url.as_str()).into_response()
        }
    }
}

mod get {
    use super::*;
    use crate::rest::auth::CompositeAuthBackend;

    pub async fn login(Query(NextUrl { next }): Query<NextUrl>) -> LoginTemplate<&'static str> {
        LoginTemplate { message: None, next }
    }

    pub async fn logout(mut auth_session: axum_login::AuthSession<CompositeAuthBackend>) -> impl IntoResponse {
        match auth_session.logout().await {
            Ok(_) => Redirect::to("/login").into_response(),
            Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }
}
