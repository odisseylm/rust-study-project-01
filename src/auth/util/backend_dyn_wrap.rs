

/// It is impossible to put impl of axum_impl::AuthBackend to pointer (Box/Rc/Arc)
/// because it implements Clone, which returns Self (!!! not pointer, on stack !!!).
/// Use wrapper AuthnBackendDynWrapperImpl/wrap_authn_backend_as_dyn WITHOUT Clone
/// if you want to put axum_login::Backend to pointer as &dyn AuthnBackendDynWrapper.
#[axum::async_trait]
#[allow(dead_code)]
pub trait AuthnBackendDynWrapper: Send + Sync {
    type User: axum_login::AuthUser;
    type Credentials: Send + Sync;
    type Error: std::error::Error + Send + Sync;
    type RealAuthnBackend: axum_login::AuthnBackend<User = Self::User, Credentials = Self::Credentials, Error = Self::Error>; // + Send + Sync;

    fn backend(&self) -> &Self::RealAuthnBackend;

    async fn authenticate(&self, creds: Self::Credentials) -> Result<Option<Self::User>, Self::Error>;
    async fn get_user(&self, user_id: &axum_login::UserId<Self::RealAuthnBackend>) -> Result<Option<Self::User>, Self::Error>;
}


pub struct AuthnBackendDynWrapperImpl <
    User: axum_login::AuthUser,
    Credentials: Send + Sync,
    Error: std::error::Error + Send + Sync,
    RealAuthnBackend: axum_login::AuthnBackend<User = User, Credentials = Credentials, Error = Error>, // + Send + Sync,
    > {
    authn_backend: RealAuthnBackend,
}

#[allow(dead_code)]
pub fn wrap_authn_backend_as_dyn<
    User: axum_login::AuthUser,
    Credentials: Send + Sync,
    Error: std::error::Error + Send + Sync,
    RealAuthnBackend: axum_login::AuthnBackend<User = User, Credentials = Credentials, Error = Error>, // + Send + Sync,
    > (authn_backend: RealAuthnBackend) -> AuthnBackendDynWrapperImpl<User, Credentials, Error, RealAuthnBackend> {
    AuthnBackendDynWrapperImpl::<User, Credentials, Error, RealAuthnBackend> { authn_backend }
}

#[axum::async_trait]
impl  <
    User: axum_login::AuthUser,
    Credentials: Send + Sync,
    Error: std::error::Error + Send + Sync,
    RealAuthnBackend: axum_login::AuthnBackend<User = User, Credentials = Credentials, Error = Error>, // + Send + Sync,
    >
    AuthnBackendDynWrapper //<Credentials = Credentials, Error = Error, RealAuthnBackend = RealAuthnBackend>
    for AuthnBackendDynWrapperImpl<User, Credentials, Error, RealAuthnBackend> {

    type User = User;
    type Credentials = Credentials;
    type Error = Error;
    type RealAuthnBackend = RealAuthnBackend;

    fn backend(&self) -> &Self::RealAuthnBackend {
        &self.authn_backend
    }

    async fn authenticate(&self, creds: Self::Credentials) -> Result<Option<Self::User>, Self::Error> {
        self.authn_backend.authenticate(creds).await
    }
    async fn get_user(&self, user_id: &axum_login::UserId<Self::RealAuthnBackend>) -> Result<Option<Self::User>, Self::Error> {
        self.authn_backend.get_user(user_id).await
    }
}


#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use crate::auth::AuthUserProviderError;
    use crate::auth::examples::auth_user::AuthUserExamplePswExtractor;
    use crate::auth::permission::bits_permission_set::IntegerBitsPermissionSet;
    use crate::auth::permission::empty_permission_provider::empty_perm_provider_arc;
    use crate::auth::permission::predefined::{Role, RolePermissionsSet};

    use super::{ AuthnBackendDynWrapperImpl, AuthnBackendDynWrapper, wrap_authn_backend_as_dyn };
    use super::super::super::{
        examples::auth_user::AuthUserExample,
        error::AuthBackendError,
        backend::AuthBackendMode,
        psw::PlainPasswordComparator,
        user_provider::InMemAuthUserProvider,
        backend::{ LoginFormAuthBackend, LoginFormAuthConfig, psw_auth::PswAuthCredentials },
    };
    type Perm = u32;
    type PermSet = IntegerBitsPermissionSet<u32>;
    use crate::util::TestResultUnwrap;

    pub fn in_memory_test_users() -> Result<InMemAuthUserProvider<AuthUserExample,Role,RolePermissionsSet,AuthUserExamplePswExtractor>, AuthUserProviderError> {
        InMemAuthUserProvider::with_users(vec!(AuthUserExample::new(1, "vovan", "qwerty")))
    }

    #[tokio::test]
    async fn test_wrap_authn_backend_as_dyn() {
        let test_users = Arc::new(in_memory_test_users().test_unwrap());
        let psw_auth = LoginFormAuthBackend::<AuthUserExample,PlainPasswordComparator,Perm,PermSet>::new(
            test_users,
            LoginFormAuthConfig { auth_mode: AuthBackendMode::AuthSupported, login_url: "/login" },
            empty_perm_provider_arc(),
        );

        use axum_login::AuthnBackend;
        let r = psw_auth.authenticate(PswAuthCredentials { username: "vovan".to_string(), password: "qwerty".to_string(), next: None }).await;
        assert!(r.is_ok());

        let as_dyn: Arc<AuthnBackendDynWrapperImpl<AuthUserExample, PswAuthCredentials, AuthBackendError, LoginFormAuthBackend<AuthUserExample,PlainPasswordComparator,Perm,PermSet>>> =
            Arc::new(wrap_authn_backend_as_dyn(psw_auth.clone()));
        let r = as_dyn.authn_backend.authenticate(PswAuthCredentials { username: "vovan".to_string(), password: "qwerty".to_string(), next: None }).await;
        assert!(r.is_ok());

        let as_dyn: Arc<dyn AuthnBackendDynWrapper<User=AuthUserExample, Credentials=PswAuthCredentials, Error=AuthBackendError, RealAuthnBackend=LoginFormAuthBackend<AuthUserExample,PlainPasswordComparator,Perm,PermSet>>> =
            Arc::new(wrap_authn_backend_as_dyn(psw_auth.clone()));
        let r = as_dyn.authenticate(PswAuthCredentials { username: "vovan".to_string(), password: "qwerty".to_string(), next: None }).await;
        assert!(r.is_ok());
    }
}
