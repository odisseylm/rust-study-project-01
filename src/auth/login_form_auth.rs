use std::sync::Arc;
use psw_auth::PswAuthCredentials;

use super::psw_auth;
use super::psw_auth::PswAuthBackendImpl;
use super::error::AuthBackendError;
use super::auth_user_provider::AuthUserProvider;
use super::auth_user::AuthUser;
use super::psw::PasswordComparator;


pub type LoginFormAuthAuthSession <
    PswComparator, // : PasswordComparator + Clone + Sync + Send,
> = axum_login::AuthSession<LoginFormAuthBackend<PswComparator>>;


#[derive(Copy, Clone, Debug)]
pub enum LoginFormAuthMode {
    // LoginFormAuthIgnored,
    LoginFormAuthSupported,
    LoginFormAuthProposed { login_form_url: Option<&'static str> },
}
// impl LoginFormAuthMode {
//     pub fn ignored(&self)->bool {
//         if let LoginFormAuthMode::LoginFormAuthIgnored = self { true }
//         else { false }
//     }
// }


#[derive(Clone)]
#[readonly::make]
pub struct LoginFormAuthBackend <
    PswComparator: PasswordComparator + Clone + Sync + Send,
> {
    psw_backend: PswAuthBackendImpl<PswComparator>,
    pub login_from_auth_mode: LoginFormAuthMode,
}

impl <
    PswComparator: PasswordComparator + Clone + Sync + Send,
> LoginFormAuthBackend<PswComparator> {
    pub fn new(
        users_provider: Arc<dyn AuthUserProvider<User = AuthUser> + Sync + Send>,
        login_from_auth_mode: LoginFormAuthMode,
    ) -> LoginFormAuthBackend<PswComparator> {
        LoginFormAuthBackend::<PswComparator> {
            psw_backend: PswAuthBackendImpl::new(users_provider.clone()),
            login_from_auth_mode,
        }
    }
}

// TODO: how to avoid duplicating this code? (probably Deref or something like that)
#[axum::async_trait]
impl <
    PswComparator: PasswordComparator + Clone + Sync + Send,
> axum_login::AuthnBackend for LoginFormAuthBackend<PswComparator> {
    type User = AuthUser;
    type Credentials = PswAuthCredentials;
    type Error = AuthBackendError;

    #[inline]
    //noinspection DuplicatedCode
    async fn authenticate(&self, creds: Self::Credentials) -> Result<Option<Self::User>, Self::Error> {
        self.psw_backend.authenticate(creds).await
    }

    #[inline]
    //noinspection DuplicatedCode
    async fn get_user(&self, user_id: &axum_login::UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        self.psw_backend.get_user(user_id).await
    }
}
