use core::fmt;
use core::fmt::Debug;
use std::sync::Arc;
use log::error;
use crate::{
    SecureString,
    error::AuthBackendError,
    user_provider::{ AuthUserProvider, AuthUserProviderError },
    psw::{ PasswordComparator },
    backend::authz_backend::{ AuthorizeBackend, PermissionProviderSource },
    permission::{
        PermissionProvider, PermissionSet,
        empty_perm_provider::{ AlwaysAllowedPermSet, EmptyPerm }
    },
};
//--------------------------------------------------------------------------------------------------


pub trait PswUser {
    fn password(&self) -> Option<SecureString>;
    //
    // 'auth' crate is designed only for authentication/authorization
    // and not designed for creating users.
    // For that reason 'password_mut' is removed now
    //
    // fn password_mut(&mut self, password: Option<SecureString>);
}


// #[derive(Clone)]
pub struct PswAuthBackendImpl <
    User: axum_login::AuthUser + PswUser,
    PermSet: PermissionSet + Clone = AlwaysAllowedPermSet<EmptyPerm>,
> {
    pub(crate) psw_comparator: Arc<dyn PasswordComparator + Send + Sync>,
    pub(crate) users_provider: Arc<dyn AuthUserProvider<User=User> + Send + Sync>,
    pub(crate) permission_provider: Arc<dyn PermissionProvider<User=User,Permission=<PermSet as PermissionSet>::Permission,PermissionSet=PermSet> + Send + Sync>,
}


impl <
    Usr: axum_login::AuthUser + PswUser,
    PermSet: PermissionSet + Clone,
> Clone for PswAuthBackendImpl<Usr,PermSet> {
    fn clone(&self) -> Self {
        PswAuthBackendImpl::<Usr,PermSet> {
            psw_comparator: Arc::clone(&self.psw_comparator),
            users_provider: Arc::clone(&self.users_provider),
            permission_provider: Arc::clone(&self.permission_provider),
        }
    }
    fn clone_from(&mut self, source: &Self) {
        self.users_provider = Arc::clone(&source.users_provider);
    }
}


impl <
    Usr: axum_login::AuthUser + PswUser,
    PermSet: PermissionSet + Clone,
> PswAuthBackendImpl<Usr,PermSet> {
    pub(crate) fn new(
        psw_comparator: Arc<dyn PasswordComparator + Send + Sync>,
        users_provider: Arc<dyn AuthUserProvider<User=Usr> + Send + Sync>,
        permission_provider: Arc<dyn PermissionProvider<User=Usr,Permission=<PermSet as PermissionSet>::Permission,PermissionSet=PermSet> + Send + Sync>,
    ) -> PswAuthBackendImpl<Usr,PermSet> {
        PswAuthBackendImpl::<Usr,PermSet> {
            psw_comparator: Arc::clone(&psw_comparator),
            users_provider: Arc::clone(&users_provider),
            permission_provider: Arc::clone(&permission_provider),
        }
    }
    pub(crate) fn users_provider(&self) -> Arc<dyn AuthUserProvider<User=Usr> + Send + Sync> {
        Arc::clone(&self.users_provider)
    }
}


#[axum::async_trait]
impl<
    Usr: axum_login::AuthUser + PswUser,
    PermSet: PermissionSet + Clone,
> axum_login::AuthnBackend for PswAuthBackendImpl<Usr,PermSet>
    where Usr: axum_login::AuthUser<Id = String>,
{
    type User = Usr;
    type Credentials = PswAuthCredentials;
    type Error = AuthBackendError;

    async fn authenticate(&self, creds: Self::Credentials) -> Result<Option<Self::User>, Self::Error> {
        let usr_res = self.get_user(&creds.username.clone()).await;

        let usr_opt = match usr_res {
            Ok(usr_opt) => usr_opt,
            Err(err) => {
                // Since it is 'layer' code it is not going to global web-app error flow and do not log error (with stack-trace).
                error!("Authentication error: {err:?}");
                return Err(err);
            }
        };

        match usr_opt {
            None => Ok(None),
            Some(usr) => {
                let usr_psw = usr.password();
                let usr_psw = usr_psw.as_ref().map(|psw|psw.as_str()).unwrap_or("");
                if !usr_psw.is_empty() && self.psw_comparator.passwords_equal(usr_psw, creds.password.as_str()) {
                    Ok(Some(usr.clone()))
                } else {
                    Ok(None)
                }
            }
        }
    }

    async fn get_user(&self, user_id: &axum_login::UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        // T O D O: what is UserId there???
        let usr_opt_res = self.users_provider.get_user_by_principal_identity(user_id).await
            .map_err(From::<AuthUserProviderError>::from);
        usr_opt_res
    }
}


#[axum::async_trait]
impl<
    Usr: axum_login::AuthUser + PswUser,
    PermSet: PermissionSet + Clone,
> PermissionProviderSource for PswAuthBackendImpl<Usr,PermSet>
    where Usr: axum_login::AuthUser<Id = String>,
{
    type User = Usr;
    type Permission = <PermSet as PermissionSet>::Permission;
    type PermissionSet = PermSet;

    #[inline]
    //noinspection DuplicatedCode
    fn permission_provider(&self)
        -> Arc<dyn PermissionProvider<User=Self::User, Permission=Self::Permission, PermissionSet=Self::PermissionSet> + Send + Sync> {
        Arc::clone(&self.permission_provider)
    }
    #[inline] // for local/non-async usage
    //noinspection DuplicatedCode
    fn permission_provider_ref<'a>(&'a self)
        -> &'a Arc<dyn PermissionProvider<User=Self::User, Permission=Self::Permission, PermissionSet=Self::PermissionSet> + Send + Sync> {
        &self.permission_provider
    }
}


#[axum::async_trait]
impl<
    Usr: axum_login::AuthUser + PswUser,
    PermSet: PermissionSet + Clone,
> AuthorizeBackend for PswAuthBackendImpl<Usr,PermSet>
    where Usr: axum_login::AuthUser<Id = String>,
{ }

#[derive(Clone, serde::Deserialize)]
pub struct PswAuthCredentials {
    pub username: String,
    pub password: SecureString,
    // seems it source/initial page... It is a bit bad design, but...
    pub next: Option<String>,
}

impl Debug for PswAuthCredentials {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AuthCredentials")
            .field("username", &self.username)
            .field("password", &"[...]")
            .field("next", &self.next)
            .finish()
    }
}
