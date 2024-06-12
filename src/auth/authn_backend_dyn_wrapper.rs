use super::auth_user;
// use super::auth_user_provider as auth;


#[axum::async_trait]
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
