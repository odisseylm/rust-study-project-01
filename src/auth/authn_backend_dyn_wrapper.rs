use super::auth_user_provider as auth;


#[axum::async_trait]
pub trait AuthnBackendDynWrapper: Send + Sync {
    type Credentials: Send + Sync;
    type Error: std::error::Error + Send + Sync;
    type RealAuthnBackend: axum_login::AuthnBackend<User = auth::AuthUser, Credentials = Self::Credentials, Error = Self::Error>; // + Send + Sync;

    async fn authenticate(&self, creds: Self::Credentials) -> Result<Option<auth::AuthUser>, Self::Error>;
    async fn get_user(&self, user_id: &axum_login::UserId<Self::RealAuthnBackend>) -> Result<Option<auth::AuthUser>, Self::Error>;
}


struct AuthnBackendDynWrapperImpl <
    Credentials: Send + Sync,
    Error: std::error::Error + Send + Sync,
    // RealAuthnBackend: axum_login::AuthnBackend<User = auth::AuthUser, Credentials = AuthnBackendDynWrapper::Credentials, Error = AuthnBackendDynWrapper::Error>, // + Send + Sync,
    RealAuthnBackend: axum_login::AuthnBackend<User = auth::AuthUser, Credentials = Credentials, Error = Error>, // + Send + Sync,
    > {
    authn_backend: RealAuthnBackend,
}

#[axum::async_trait]
impl  <
    Credentials: Send + Sync,
    Error: std::error::Error + Send + Sync,
    RealAuthnBackend: axum_login::AuthnBackend<User = auth::AuthUser, Credentials = Credentials, Error = Error>, // + Send + Sync,
    >
    AuthnBackendDynWrapper //<Credentials = Credentials, Error = Error, RealAuthnBackend = RealAuthnBackend>
    for AuthnBackendDynWrapperImpl<Credentials, Error, RealAuthnBackend> {

    type Credentials = Credentials;
    type Error = Error;
    type RealAuthnBackend = RealAuthnBackend;

    async fn authenticate(&self, creds: Self::Credentials) -> Result<Option<auth::AuthUser>, Self::Error> {
        self.authn_backend.authenticate(creds).await
    }
    async fn get_user(&self, user_id: &axum_login::UserId<Self::RealAuthnBackend>) -> Result<Option<auth::AuthUser>, Self::Error> {
        self.authn_backend.get_user(user_id).await
    }
}
