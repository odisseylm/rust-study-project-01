use core::fmt;
use core::marker::PhantomData;
use std::sync::Arc;

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
    PswComparator: PasswordComparator + Clone + Sync + Send,
> {
    users_provider: Arc<dyn AuthUserProvider<User=User> + Sync + Send>,
    _pd: PhantomData<PswComparator>,
}


impl <
    User: axum_login::AuthUser + PswUser,
    PswComparator: PasswordComparator + Clone + Sync + Send,
> Clone for PswAuthBackendImpl<User,PswComparator> {
    fn clone(&self) -> Self {
        PswAuthBackendImpl::<User,PswComparator> {
            users_provider: self.users_provider.clone(),
            _pd: PhantomData,
        }
    }
    fn clone_from(&mut self, source: &Self) {
        self.users_provider = source.users_provider.clone();
    }
}


impl <
    User: axum_login::AuthUser + PswUser,
    PswComparator: PasswordComparator + Clone + Sync + Send,
> PswAuthBackendImpl<User,PswComparator> {
    pub fn new(
        users_provider: Arc<dyn AuthUserProvider<User=User> + Sync + Send>,
    ) -> PswAuthBackendImpl<User,PswComparator> {
        PswAuthBackendImpl::<User,PswComparator> {
            users_provider: users_provider.clone(),
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
    PswComparator: PasswordComparator + Clone + Sync + Send,
> axum_login::AuthnBackend for PswAuthBackendImpl<User,PswComparator>
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

#[derive(Clone, serde::Deserialize)]
pub struct PswAuthCredentials {
    pub username: String,
    pub password: String,
    // seems it source/initial page... It is a bit bad design, but...
    pub next: Option<String>,
}

impl fmt::Debug for PswAuthCredentials {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AuthCredentials")
            .field("username", &self.username)
            .field("password", &"[...]")
            .field("next", &self.next)
            .finish()
    }
}
