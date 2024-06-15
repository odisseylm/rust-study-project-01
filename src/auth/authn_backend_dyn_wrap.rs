use super::auth_user;


/// It is impossible to put impl of axum_impl::AuthBackend to pointer (Box/Rc/Arc)
/// because it implements Clone, which returns Self (!!! not pointer, on stack !!!).
/// Use wrapper AuthnBackendDynWrapperImpl/wrap_authn_backend_as_dyn WITHOUT Clone
/// if you want to put axum_login::Backend to pointer as &dyn AuthnBackendDynWrapper.
#[axum::async_trait]
#[allow(dead_code)]
pub trait AuthnBackendDynWrapper: Send + Sync {
    type Credentials: Send + Sync;
    type Error: std::error::Error + Send + Sync;
    type RealAuthnBackend: axum_login::AuthnBackend<User = auth_user::AuthUser, Credentials = Self::Credentials, Error = Self::Error>; // + Send + Sync;

    fn backend(&self) -> &Self::RealAuthnBackend;

    async fn authenticate(&self, creds: Self::Credentials) -> Result<Option<auth_user::AuthUser>, Self::Error>;
    async fn get_user(&self, user_id: &axum_login::UserId<Self::RealAuthnBackend>) -> Result<Option<auth_user::AuthUser>, Self::Error>;
}


pub struct AuthnBackendDynWrapperImpl <
    Credentials: Send + Sync,
    Error: std::error::Error + Send + Sync,
    RealAuthnBackend: axum_login::AuthnBackend<User = auth_user::AuthUser, Credentials = Credentials, Error = Error>, // + Send + Sync,
    > {
    authn_backend: RealAuthnBackend,
}

#[allow(dead_code)]
pub fn wrap_authn_backend_as_dyn<
    Credentials: Send + Sync,
    Error: std::error::Error + Send + Sync,
    RealAuthnBackend: axum_login::AuthnBackend<User = auth_user::AuthUser, Credentials = Credentials, Error = Error>, // + Send + Sync,
    > (authn_backend: RealAuthnBackend) -> AuthnBackendDynWrapperImpl<Credentials, Error, RealAuthnBackend> {
    AuthnBackendDynWrapperImpl::<Credentials, Error, RealAuthnBackend> { authn_backend }
}

#[axum::async_trait]
impl  <
    Credentials: Send + Sync,
    Error: std::error::Error + Send + Sync,
    RealAuthnBackend: axum_login::AuthnBackend<User = auth_user::AuthUser, Credentials = Credentials, Error = Error>, // + Send + Sync,
    >
    AuthnBackendDynWrapper //<Credentials = Credentials, Error = Error, RealAuthnBackend = RealAuthnBackend>
    for AuthnBackendDynWrapperImpl<Credentials, Error, RealAuthnBackend> {

    type Credentials = Credentials;
    type Error = Error;
    type RealAuthnBackend = RealAuthnBackend;

    fn backend(&self) -> &Self::RealAuthnBackend {
        &self.authn_backend
    }

    async fn authenticate(&self, creds: Self::Credentials) -> Result<Option<auth_user::AuthUser>, Self::Error> {
        self.authn_backend.authenticate(creds).await
    }
    async fn get_user(&self, user_id: &axum_login::UserId<Self::RealAuthnBackend>) -> Result<Option<auth_user::AuthUser>, Self::Error> {
        self.authn_backend.get_user(user_id).await
    }
}


#[cfg(test)]
mod tests {
    use super::{ AuthnBackendDynWrapperImpl, AuthnBackendDynWrapper, wrap_authn_backend_as_dyn };
    use std::sync::Arc;
    use axum_login::AuthnBackend;
    use crate::auth;
    use crate::auth::{ psw_auth, InMemAuthUserProvider, psw_auth::{BasicAuthMode, LoginFormMode}, PlainPasswordComparator };
    use crate::util::TestResultUnwrap;

    #[tokio::test]
    async fn test_wrap_authn_backend_as_dyn() {
        let test_users = Arc::new(InMemAuthUserProvider::test_users().test_unwrap());
        let psw_auth = psw_auth::AuthBackend::<PlainPasswordComparator>::new(test_users, BasicAuthMode::BasicAuthSupported, LoginFormMode::LoginFormSupported);

        let r = psw_auth.authenticate(psw_auth::AuthCredentials { username: "vovan".to_string(), password: "qwerty".to_string(), next: None }).await;
        assert!(r.is_ok());

        let as_dyn: Arc<AuthnBackendDynWrapperImpl<psw_auth::AuthCredentials, auth::AuthBackendError, psw_auth::AuthBackend<PlainPasswordComparator>>> =
            Arc::new(wrap_authn_backend_as_dyn(psw_auth.clone()));
        let r = as_dyn.authn_backend.authenticate(psw_auth::AuthCredentials { username: "vovan".to_string(), password: "qwerty".to_string(), next: None }).await;
        assert!(r.is_ok());

        let as_dyn: Arc<dyn AuthnBackendDynWrapper<Credentials=psw_auth::AuthCredentials, Error=auth::AuthBackendError, RealAuthnBackend=psw_auth::AuthBackend<PlainPasswordComparator>>> =
            Arc::new(wrap_authn_backend_as_dyn(psw_auth.clone()));
        let r = as_dyn.authenticate(psw_auth::AuthCredentials { username: "vovan".to_string(), password: "qwerty".to_string(), next: None }).await;
        assert!(r.is_ok());
    }
}
