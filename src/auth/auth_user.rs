use core::fmt;
use super::psw::PasswordComparator;


#[derive(Clone)]
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(sqlx::FromRow)]
#[readonly::make]
pub struct AuthUser {
    pub id: i64,
    pub username: String,
    pub password: Option<String>,
    pub access_token: Option<String>,
}

impl AuthUser {
    pub fn new(id: i64, username: &'static str, password: &'static str) -> AuthUser {
        AuthUser { id, username: username.to_string(), password: Some(password.to_string()), access_token: None }
    }
    pub fn access_token(&mut self, access_token: Option<String>) {
        self.access_token = access_token;
    }
    pub fn has_password<PswComparator: PasswordComparator>(&self, cred_psw: &str) -> bool {
        match self.password {
            None => false,
            Some(ref usr_psw) =>
                 PswComparator::passwords_equal(usr_psw, cred_psw),
        }
    }
}


impl fmt::Debug for AuthUser {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("User0")
            .field("username", &self.username)
            .field("psw", &"[...]")
            .field("access_token", &"[...]")
            .finish()
    }
}

impl axum_login::AuthUser for AuthUser {
    type Id = i64;

    fn id(&self) -> Self::Id {
        self.id
    }
    fn session_auth_hash(&self) -> &[u8] {
        if let Some(access_token) = &self.access_token {
            return access_token.as_bytes();
        }

        if let Some(password) = &self.password {
            // ???
            // We use the password hash as the auth hash -> what this means
            // is when the user changes their password the auth session becomes invalid.
            //
            return password.as_bytes();
        }

        &[]
    }
}

/*
#[derive(Debug, Clone)]
struct AuthRequestData {
    original_uri: Option<axum::extract::OriginalUri>,
    basic_auth: Option<axum_extra::headers::authorization::Basic>,
}

#[async_trait::async_trait]
impl<S> axum::extract::FromRequestParts<S> for AuthRequestData where S: Send + Sync {
    type Rejection = core::convert::Infallible;

    async fn from_request_parts(parts: &mut http::request::Parts, state: &S) -> Result<Self, Self::Rejection> {
        use axum::extract::OriginalUri;
        use axum_extra:: { TypedHeader, typed_header::TypedHeaderRejection, headers::{ Authorization, authorization::Basic } };

        let original_uri: Option<OriginalUri> = OriginalUri::from_request_parts(parts, state).await.ok();

        let basic_auth: Result<TypedHeader<Authorization<Basic>>, TypedHeaderRejection> =
            TypedHeader::<Authorization::<Basic>>::from_request_parts(parts, state).await;
        let basic_auth: Option<Basic> =
            if let Ok(TypedHeader(Authorization(basic_auth))) = basic_auth { Some(basic_auth) }
            else { None };

        Ok(AuthRequestData {
            original_uri,
            basic_auth,
        })

        // let extracted_basic_ath = req.extensions().get::<TypedHeader<AuthorizationHeader<Basic>>>();
    }
}
*/
