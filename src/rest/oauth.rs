
use axum::{
    extract::Query,
    http::StatusCode,
    response::{IntoResponse, Redirect},
    routing::get as GET,
    Router,
};
use axum_login::tower_sessions::Session;
use serde::Deserialize;

use crate::web::auth::{ LoginTemplate, NEXT_URL_KEY };
// use super::auth::AuthSession;

pub const CSRF_STATE_KEY: &str = "oauth.csrf-state";

#[derive(Debug, Clone, Deserialize)]
pub struct AuthzResp {
    code: String,
    state: oauth2::CsrfToken,
}

pub fn router() -> Router<()> {
    Router::new().route("/oauth/callback", GET(get::callback))
}

mod get {
    use crate::auth::composite_auth::CompositeAuthCredentials;
    // use axum_login::AuthSession;
    use super::super::auth::AuthSession;
    use crate::auth::oauth2_auth::{ OAuth2AuthCredentials as OAuthCreds };
    use super::*;

    pub async fn callback /*<B: axum_login::AuthnBackend>*/ (
        // mut auth_session: AuthSession<B>,
        mut auth_session: AuthSession,
        session: Session,
        Query(AuthzResp {
                  code,
                  state: new_state,
              }): Query<AuthzResp>,
    ) -> impl IntoResponse {
        let Ok(Some(old_state)) = session.get(CSRF_STATE_KEY).await else {
            return StatusCode::BAD_REQUEST.into_response();
        };

        let creds = OAuthCreds { code, old_state, new_state, };

        let user = match auth_session.authenticate(CompositeAuthCredentials::OAuth(creds)).await {
            Ok(Some(user)) => user,
            Ok(None) => {
                return (
                    StatusCode::UNAUTHORIZED,
                    LoginTemplate {
                        message: Some("Invalid CSRF state.".to_string()),
                        next: None,
                    },
                )
                    .into_response()
            }
            Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        };

        if auth_session.login(&user).await.is_err() {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }

        if let Ok(Some(next)) = session.remove::<String>(NEXT_URL_KEY).await {
            Redirect::to(&next).into_response()
        } else {
            Redirect::to("/").into_response()
        }
    }
}
