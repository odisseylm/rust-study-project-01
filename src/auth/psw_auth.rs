use core::fmt;
use core::marker::PhantomData;
use std::sync::Arc;

use axum_login;
use axum_login::tower_sessions;
use axum::body::Body;
use axum_extra::TypedHeader;
use axum_extra::headers::Authorization as AuthorizationHeader;
use axum_extra::headers::authorization::Basic;

use super::error::AuthBackendError;
use super::auth_user_provider::{ AuthUserProvider, AuthUserProviderError };
use super::auth_user::AuthUser;
use super::psw::PasswordComparator;



async fn is_authenticated <
    PswComparator: PasswordComparator + Clone + Sync + Send,
> (
    auth_session: AuthSession<PswComparator>,
    basic_auth_creds: Option<TypedHeader<AuthorizationHeader<Basic>>>,
) -> bool {
    auth_session.backend.is_authenticated(&auth_session.user, &basic_auth_creds).await
}


#[inline] // TODO: remove from there
pub async fn validate_auth_temp <
    PC: PasswordComparator + Clone + Sync + Send,
    > (
    auth_session: AuthSession<PC>, basic_auth_creds: Option<TypedHeader<AuthorizationHeader<Basic>>>,
    req: axum::extract::Request, next: axum::middleware::Next) -> http::Response<Body> {
    validate_auth_by_password(auth_session, basic_auth_creds, req, next).await
}


pub async fn validate_auth_by_password<
    PC: PasswordComparator + Clone + Sync + Send,
    >(
    auth_session: AuthSession<PC>,
    basic_auth_creds: Option<TypedHeader<AuthorizationHeader<Basic>>>,
    req: axum::extract::Request,
    next: axum::middleware::Next
    ) -> http::Response<Body> {

    if is_authenticated(auth_session, basic_auth_creds).await {
        next.run(req).await
    } else {
        // or redirect to login page
        // should be configurable outside: dev or prod
        crate::rest::error_rest::unauthenticated_401_response()
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
    LoginFormProposed { login_form_url: &'static str },
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
        basic_auth_creds: &Option<TypedHeader<AuthorizationHeader<Basic>>>,
    ) -> bool {

        if !self.login_form_mode.ignored() && auth_session_user.is_some() {
            return true;
        }

        if !self.basic_auth_mode.ignored() {
            if let Some(TypedHeader(AuthorizationHeader(ref creds))) = basic_auth_creds {
                // T O D O: avoid to_string()
                use axum_login::AuthnBackend;
                let is_auth_res = self.authenticate(AuthCredentials {
                    username: creds.username().to_string(),
                    password: creds.password().to_string(),
                    next: None,
                }).await;

                return is_auth_res.is_ok();
            }
        };

        return false;
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
                // if usr.username == creds.username && usr.psw == creds.password {
                if usr.username == creds.username &&
                    PswComparator::passwords_equal(usr_psw, creds.password.as_str()) {
                    Ok(Some(usr.clone()))
                } else {
                    Err(Self::Error::IncorrectUsernameOrPsw)
                }
            }
        }
    }

    async fn get_user(&self, user_id: &axum_login::UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        // TODO: what is UserId there???
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
