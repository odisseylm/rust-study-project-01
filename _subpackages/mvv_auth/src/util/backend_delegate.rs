

#[allow(dead_code)]
trait AuthBackendContainer {
    type User: axum_login::AuthUser;
    type Credentials: Send + Sync;
    type Error: std::error::Error + Send + Sync;
    type AuthnBackend: axum_login::AuthnBackend<User=Self::User, Credentials=Self::Credentials, Error=Self::Error>;

    fn nested_auth_backend<'a>() -> &'a Self::AuthnBackend;
}


// !!! It does NOT work - it requires 'dyn', what is impossible for axum_login::AuthnBackend/Clone impl !!!

/*
#[axum::async_trait]
impl <
    User: axum_login::AuthUser + Send + Sync,
    Credentials: Send + Sync,
    Error: std::error::Error + Send + Sync,
    AuthnBackend: axum_login::AuthnBackend<User=User, Credentials=Credentials, Error=Error> + Clone + Send + Sync,
    // PswComparator: PasswordComparator + Clone + Sync + Send,
> axum_login::AuthnBackend for AuthBackendContainer<User=User, Credentials=Credentials, Error=Error, AuthnBackend=AuthnBackend> {
    type User = User;
    type Credentials = Credentials;
    type Error = Error;

    async fn authenticate(&self, creds: Self::Credentials) -> Result<Option<Self::User>, Self::Error>;
    async fn get_user(&self, user_id: &axum_login::UserId<Self>) -> Result<Option<Self::User>, Self::Error>;
    //
    // #[inline]
    // async fn authenticate(&self, creds: Self::Credentials) -> Result<Option<Self::User>, Self::Error> {
    //     self.nested_auth_backend().authenticate(creds).await
    // }
    // #[inline]
    // async fn get_user(&self, user_id: &axum_login::UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
    //     self.nested_auth_backend().get_user(user_id).await
    // }
}
*/
