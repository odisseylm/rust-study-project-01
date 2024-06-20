use core::fmt;
use core::fmt::Debug;
use core::marker::PhantomData;
use std::hash::Hash;
use std::sync::Arc;
use crate::auth::backend::authz_backend::{ AuthorizeBackend, PermissionProviderSource};
use crate::auth::permission::{ PermissionProvider, PermissionSet };

use super::super::{
    error::AuthBackendError,
    user_provider::{ AuthUserProvider, AuthUserProviderError },
    psw::PasswordComparator,
};


pub trait PswUser {
    fn password(&self) -> Option<String>;
    fn password_mut(&mut self, password: Option<String>);
}


// #[derive(Clone)]
pub struct PswAuthBackendImpl <
    User: axum_login::AuthUser + PswUser,
    PswComparator: PasswordComparator + Debug + Clone + Sync + Send,
    Perm: Hash + Eq + Debug + Clone + Send + Sync,
    PermSet: PermissionSet<Permission=Perm> + Debug + Clone + Send + Sync
> {
    users_provider: Arc<dyn AuthUserProvider<User=User> + Sync + Send>,
    permission_provider: Arc<dyn PermissionProvider<User=User,Permission=Perm,PermissionSet=PermSet> + Sync + Send>,
    _pd: PhantomData<PswComparator>,
}


impl <
    User: axum_login::AuthUser + PswUser,
    PswComparator: PasswordComparator + Debug + Clone + Sync + Send,
    Perm: Hash + Eq + Debug + Clone + Send + Sync,
    PermSet: PermissionSet<Permission=Perm> + Debug + Clone + Send + Sync
> Clone for PswAuthBackendImpl<User,PswComparator,Perm,PermSet> {
    fn clone(&self) -> Self {
        PswAuthBackendImpl::<User,PswComparator,Perm,PermSet> {
            users_provider: self.users_provider.clone(),
            permission_provider: self.permission_provider.clone(),
            _pd: PhantomData,
        }
    }
    fn clone_from(&mut self, source: &Self) {
        self.users_provider = source.users_provider.clone();
    }
}


impl <
    User: axum_login::AuthUser + PswUser,
    PswComparator: PasswordComparator + Debug + Clone + Sync + Send,
    Perm:  Hash + Eq + Debug + Clone +Send + Sync,
    PermSet: PermissionSet<Permission=Perm> + Debug + Clone + Send + Sync
> PswAuthBackendImpl<User,PswComparator,Perm,PermSet> {
    pub fn new(
        users_provider: Arc<dyn AuthUserProvider<User=User> + Sync + Send>,
        permission_provider: Arc<dyn PermissionProvider<User=User,Permission=Perm,PermissionSet=PermSet> + Sync + Send>,
    ) -> PswAuthBackendImpl<User,PswComparator,Perm,PermSet> {
        PswAuthBackendImpl::<User,PswComparator,Perm,PermSet> {
            users_provider: users_provider.clone(),
            permission_provider: permission_provider.clone(),
            _pd: PhantomData,
        }
    }
    pub(crate) fn users_provider(&self) -> Arc<dyn AuthUserProvider<User=User> + Sync + Send> {
        self.users_provider.clone()
    }
}


#[axum::async_trait]
impl<
    User: axum_login::AuthUser + PswUser,
    PswComparator: PasswordComparator + Debug + Clone + Sync + Send,
    Perm: Hash + Eq + Debug + Clone + Send + Sync,
    PermSet: PermissionSet<Permission=Perm> + Debug + Clone + Send + Sync
> axum_login::AuthnBackend for PswAuthBackendImpl<User,PswComparator,Perm,PermSet>
    where User: axum_login::AuthUser<Id = String>,
{
    type User = User;
    type Credentials = PswAuthCredentials;
    type Error = AuthBackendError;

    async fn authenticate(&self, creds: Self::Credentials) -> Result<Option<Self::User>, Self::Error> {
        let usr_res = self.users_provider.get_user_by_principal_identity(&creds.username.clone()).await;

        let usr_opt = match usr_res {
            Ok(usr_opt) => usr_opt,
            Err(err) => return Err(Self::Error::UserProviderError(err))
        };

        match usr_opt {
            None => Ok(None),
            Some(usr) => {
                let usr_psw = usr.password().unwrap_or("".to_string());
                if !usr_psw.is_empty() && PswComparator::passwords_equal(usr_psw.as_str(), creds.password.as_str()) {
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
    User: axum_login::AuthUser + PswUser,
    PswComparator: PasswordComparator + Debug + Clone + Sync + Send,
    Perm: Hash + Eq + Debug + Clone + Send + Sync,
    PermSet: PermissionSet<Permission=Perm> + Debug + Clone + Send + Sync
> PermissionProviderSource for PswAuthBackendImpl<User,PswComparator,Perm,PermSet>
    where User: axum_login::AuthUser<Id = String>,
{
    type User = User;
    type Permission = Perm;
    type PermissionSet = PermSet;

    fn permission_provider(&self) -> Arc<dyn PermissionProvider<User=Self::User, Permission=Self::Permission, PermissionSet=Self::PermissionSet>> {
        self.permission_provider.clone()
    }
    // TODO: try to use ref
    // fn permission_provider_ref(&self) -> &Arc<dyn PermissionProvider<User=Self::User, Permission=Self::Permission, PermissionSet=Self::PermissionSet>> {
    //     &self.permission_provider
    // }
}


#[axum::async_trait]
impl<
    User: axum_login::AuthUser + PswUser,
    PswComparator: PasswordComparator + Debug + Clone + Sync + Send,
    Perm: Hash + Eq + Debug + Clone + Send + Sync,
    PermSet: PermissionSet<Permission=Perm> + Debug + Clone + Send + Sync
> AuthorizeBackend for PswAuthBackendImpl<User,PswComparator,Perm,PermSet>
    where User: axum_login::AuthUser<Id = String>,
{
    // type User = User;
    // type Permission = Perm;
    // type PermissionSet = PermSet;

    /*
    async fn authorize(&self, user: &Self::User, required_permissions: Self::PermissionSet)
        -> Result<AuthorizationResult<Perm,PermSet>, PermissionProcessError> {

        let user_perms = self.permission_provider().get_all_permissions(user).await ?;
        let authz_res: AuthorizationResult<Perm,PermSet> =
            user_perms.verify_required_permissions(required_permissions) ?.into();
        Ok(authz_res)
    }
    async fn has_permission(&self, user: &Self::User, required_permission: Self::Permission) -> Result<bool, PermissionProcessError> {
        let authz_res: AuthorizationResult<Perm,PermSet> = self.authorize(user,
            PermissionSet::from_permission(required_permission)).await ?;
        Ok(authz_res.is_authorized())
    }
    async fn has_permissions(&self, user: &Self::User, required_permissions: Self::PermissionSet) -> Result<bool, PermissionProcessError> {
        let authz_res: AuthorizationResult<Perm,PermSet> =
            self.authorize(user, required_permissions).await ?;
        Ok(authz_res.is_authorized())
    }
    */
}

#[derive(Clone, serde::Deserialize)]
pub struct PswAuthCredentials {
    pub username: String,
    pub password: String,
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
