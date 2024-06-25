
use askama::Template;
use axum::{
    extract::Query,
    http::StatusCode,
    response::{IntoResponse, Redirect},
    routing::{get as GET, post as POST},
    Form, Router,
};
use axum_login::tower_sessions::Session;
use serde::Deserialize;


pub const NEXT_URL_KEY: &str = "auth.next-url";

#[derive(Template)]
#[template(path = "login.html")]
pub struct LoginTemplate {
    pub message: Option<String>,
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
        use crate::rest::auth::AuthUser;
        use mvv_auth::backend::PswAuthCredentials as PasswordCreds;
        use mvv_auth::examples::composite_auth::{CompositeAuthCredentials, CompositeAuthnBackendExample};
        use crate::rest::oauth::CSRF_STATE_KEY;
        use super::*;

        pub async fn password(
            mut auth_session: axum_login::AuthSession<CompositeAuthnBackendExample>,
            Form(creds): Form<PasswordCreds>,
        ) -> impl IntoResponse {
            let auth_res: Result<Option<AuthUser>, axum_login::Error<CompositeAuthnBackendExample>> =
                auth_session.authenticate(
                    CompositeAuthCredentials::Password(creds.clone())).await;
            let user = match auth_res {
                Ok(Some(user)) => user,
                Ok(None) => {
                    return LoginTemplate {
                            message: Some("Invalid credentials.".to_string()),
                            next: creds.next,
                        }
                        .into_response()
                }
                Err(err) => {
                    match err {
                        axum_login::Error::Session(err) => {
                            let err2 = err.to_string();
                            println!("{}", err2);
                            error!("Authentication session error [{}]", err)
                        }
                        axum_login::Error::Backend(err) => {
                            let err2 = err.to_string();
                            println!("{}", err2);
                            match err {
                                // AuthBackendError::NoUser | AuthBackendError::IncorrectUsernameOrPsw => {
                                //     return LoginTemplate {
                                //             message: Some("Invalid credentials.".to_string()),
                                //             next: creds.next,
                                //         }
                                //         .into_response()
                                // }
                                // AuthBackendError::UserProviderError(_) => {}
                                // AuthBackendError::Sqlx(_) => {}
                                // AuthBackendError::Reqwest(_) => {}
                                // AuthBackendError::OAuth2(_) => {}
                                // AuthBackendError::NoRequestedBackend => {}
                                // AuthBackendError::TaskJoin(_) => {}
                                // AuthBackendError::ConfigError(_) => {}
                                err => {
                                    error!("Authentication backend error [{}]", err)
                                }
                            }
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
            auth_session: axum_login::AuthSession<CompositeAuthnBackendExample>,
            session: Session,
            Form(NextUrl { next }): Form<NextUrl>,
        ) -> impl IntoResponse {
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
    use mvv_auth::examples::composite_auth::CompositeAuthnBackendExample;
    use super::*;

    pub async fn login(Query(NextUrl { next }): Query<NextUrl>) -> LoginTemplate {
        LoginTemplate { message: None, next }
    }

    pub async fn logout(mut auth_session: axum_login::AuthSession<CompositeAuthnBackendExample>) -> impl IntoResponse {
        match auth_session.logout().await {
            Ok(_) => Redirect::to("/login").into_response(),
            Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }
}
