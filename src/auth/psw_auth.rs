use core::fmt;
use core::marker::PhantomData;
use std::sync::Arc;
use askama_axum::IntoResponse;

use axum_login;
use axum_login::tower_sessions;
use axum::body::Body;
use axum::extract::OriginalUri;
use axum_extra::TypedHeader;
use axum_extra::headers::Authorization as AuthorizationHeader;
use axum_extra::headers::authorization::Basic;
use crate::auth::UnauthenticatedAction;

use super::error::AuthBackendError;
use super::auth_user_provider::{ AuthUserProvider, AuthUserProviderError };
use super::auth_user::AuthUser;
use super::psw::PasswordComparator;



async fn is_authenticated <
    PswComparator: PasswordComparator + Clone + Sync + Send,
> (
    auth_session: AuthSession<PswComparator>,
    original_uri: OriginalUri,
    basic_auth_creds: Option<TypedHeader<AuthorizationHeader<Basic>>>,
) -> Result<(), UnauthenticatedAction> {
    auth_session.backend.is_authenticated(&auth_session.user, &original_uri, &basic_auth_creds).await
}


pub async fn validate_auth_by_password<
    PC: PasswordComparator + Clone + Sync + Send,
    >(
    auth_session: AuthSession<PC>,
    original_uri: OriginalUri,
    basic_auth_creds: Option<TypedHeader<AuthorizationHeader<Basic>>>,
    req: axum::extract::Request,
    next: axum::middleware::Next
    ) -> http::Response<Body> {

    let is_auth_res = is_authenticated(auth_session, original_uri, basic_auth_creds).await;
    match is_auth_res {
        Ok(_) => next.run(req).await,
        Err(action) => action.into_response()
    }
}

pub type AuthSession <
    PswComparator, // : PasswordComparator + Clone + Sync + Send,
> = axum_login::AuthSession<AuthBackend<PswComparator>>;


/*
pub fn test_auth_by_psw_manager_layer()
    -> axum_login::AuthManagerLayer<AuthBackend<PlainPasswordComparator>, tower_sessions::MemoryStore> {

    use tower_sessions::{ MemoryStore, service::SessionManagerLayer, Expiry };

    let user_provider = Arc::new(InMemAuthUserProvider::new());
    let session_layer = SessionManagerLayer::new(MemoryStore::default())
        .with_secure(false)
        .with_same_site(SameSite::Lax) // Ensure we send the cookie from the OAuth redirect.
        .with_expiry(Expiry::OnInactivity(Duration::days(1)));

    build_auth_by_psw_manager_layer(user_provider, session_layer)
}
*/


pub fn build_auth_by_psw_manager_layer <
    PswComparator: PasswordComparator + Clone + Sync + Send,
    SessionStore: tower_sessions::SessionStore,
    CookieController: tower_sessions::service::CookieController,
> (
    basic_auth_mode: BasicAuthMode,
    login_form_mode: LoginFormMode,
    user_provider: Arc<dyn AuthUserProvider<User = AuthUser> + Sync + Send>,
    session_manager_layer: tower_sessions::SessionManagerLayer<SessionStore, CookieController>,
) -> axum_login::AuthManagerLayer<AuthBackend<PswComparator>, SessionStore, CookieController> {

    // Auth service.
    //
    // This combines the session layer with our backend to establish the auth
    // service which will provide the auth session as a request extension.
    let backend: AuthBackend<PswComparator> = AuthBackend::new(user_provider, basic_auth_mode, login_form_mode);
    axum_login::AuthManagerLayerBuilder::new(backend, session_manager_layer).build()
}


#[derive(Copy, Clone, Debug)]
pub enum BasicAuthMode {
    BasicAuthIgnored,
    BasicAuthSupported,
    BasicAuthProposed,
}
impl BasicAuthMode {
    pub fn ignored(&self)->bool {
        if let BasicAuthMode::BasicAuthIgnored = self { true }
        else { false }
    }
}


#[derive(Copy, Clone, Debug)]
pub enum LoginFormMode {
    LoginFormIgnored,
    LoginFormSupported,
    LoginFormProposed { login_form_url: Option<&'static str> },
}
impl LoginFormMode {
    pub fn ignored(&self)->bool {
        if let LoginFormMode::LoginFormIgnored = self { true }
        else { false }
    }
}


// #[derive(Clone)]
pub struct AuthBackend <
    PswComparator: PasswordComparator + Clone + Sync + Send,
> {
    users_provider: Arc<dyn AuthUserProvider<User = AuthUser> + Sync + Send>,
    basic_auth_mode: BasicAuthMode,
    login_form_mode: LoginFormMode,
    _pd: PhantomData<PswComparator>,
}

impl  <
    PswComparator: PasswordComparator + Clone + Sync + Send,
> AuthBackend <PswComparator> {

    pub async fn is_authenticated (
        &self,
        auth_session_user: &Option<AuthUser>,
        original_uri: &OriginalUri, // TODO: try to remove
        basic_auth_creds: &Option<TypedHeader<AuthorizationHeader<Basic>>>, // TODO: try to remove
    ) -> Result<(), UnauthenticatedAction> {

        if !self.basic_auth_mode.ignored() {
            if let Some(TypedHeader(AuthorizationHeader(ref creds))) = basic_auth_creds {
                // T O D O: avoid to_string()
                use axum_login::AuthnBackend;
                let is_auth_res = self.authenticate(AuthCredentials {
                    username: creds.username().to_string(),
                    password: creds.password().to_string(),
                    next: None,
                }).await;

                if is_auth_res.is_ok() { return Ok(()) }
            }
        };

        if !self.login_form_mode.ignored() && auth_session_user.is_some() {
            return Ok(());
        }

        if let BasicAuthMode::BasicAuthProposed = self.basic_auth_mode {
            return Err(UnauthenticatedAction::ProposeBase64)
        };
        if let LoginFormMode::LoginFormProposed { login_form_url } = self.login_form_mode {
            return Err(UnauthenticatedAction::ProposeLoginForm { login_form_url, initial_url: Some(original_uri.to_string()) })
        };

        return Err(UnauthenticatedAction::NoAction)
    }
}

impl<
    PswComparator: PasswordComparator + Clone + Sync + Send,
> Clone for AuthBackend<PswComparator> {
    fn clone(&self) -> Self {
        AuthBackend::<PswComparator> {
            users_provider: self.users_provider.clone(),
            basic_auth_mode: self.basic_auth_mode,
            login_form_mode: self.login_form_mode,
            _pd: PhantomData,
        }
    }
    fn clone_from(&mut self, source: &Self) {
        self.users_provider = source.users_provider.clone();
    }
}


impl<PswComparator> AuthBackend<PswComparator> where
    PswComparator: PasswordComparator + Clone + Sync + Send,
{
    pub fn new(
        users_provider: Arc<dyn AuthUserProvider<User = AuthUser> + Sync + Send>,
        basic_auth_mode: BasicAuthMode,
        login_form_mode: LoginFormMode,
    ) -> AuthBackend<PswComparator> {
        AuthBackend::<PswComparator> {
            users_provider: users_provider.clone(),
            _pd: PhantomData,
            basic_auth_mode,
            login_form_mode,
        }
    }
}



#[axum::async_trait]
impl<
    PswComparator: PasswordComparator + Clone + Sync + Send,
    > axum_login::AuthnBackend for AuthBackend<PswComparator> {
    type User = AuthUser;
    type Credentials = AuthCredentials;
    type Error = AuthBackendError;

    async fn authenticate(&self, creds: Self::Credentials) -> Result<Option<Self::User>, Self::Error> {
        let usr_res = self.users_provider.get_user_by_name(creds.username.as_str()).await;

        let usr_opt = match usr_res {
            Ok(usr_opt) => usr_opt,
            Err(err) => return Err(Self::Error::UserProviderError(err))
        };

        match usr_opt {
            None => Err(Self::Error::NoUser),
            Some(usr) => {
                let usr_psw = usr.password.as_ref().map(|s|s.as_str()).unwrap_or("");
                if !usr_psw.is_empty() && PswComparator::passwords_equal(usr_psw, creds.password.as_str()) {
                    Ok(Some(usr.clone()))
                } else {
                    Err(Self::Error::IncorrectUsernameOrPsw)
                }
            }
        }
    }

    async fn get_user(&self, user_id: &axum_login::UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        // T O D O: what is UserId there???
        let usr_opt_res = self.users_provider.get_user_by_id(user_id).await
            .map_err(From::<AuthUserProviderError>::from);
        usr_opt_res
    }
}

#[derive(Clone, serde::Deserialize)]
pub struct AuthCredentials {
    pub username: String,
    pub password: String,
    // seems it source/initial page... It is a bit bad design, but...
    pub next: Option<String>,
}

impl fmt::Debug for AuthCredentials {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Cred0 {{ username: {:?}, psw: [...] }},", self.username)
    }
}
