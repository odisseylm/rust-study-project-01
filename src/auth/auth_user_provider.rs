use core::fmt;
use std::sync::Arc;
use super::auth_user::AuthUser;


#[axum::async_trait]
pub trait AuthUserProvider : fmt::Debug {
    type User: axum_login::AuthUser;
    async fn get_user_by_name(&self, username: &str) -> Result<Option<Self::User>, AuthUserProviderError>;
    async fn get_user_by_id(&self, user_id: &<AuthUser as axum_login::AuthUser>::Id) -> Result<Option<Self::User>, AuthUserProviderError>;
}



#[derive(Debug)]
struct AuthUserProviderStaticTypeArcWrapper<
    User: axum_login::AuthUser,
    UsrProviderDelegate: AuthUserProvider<User=User> + Send + Sync,
> where UsrProviderDelegate: fmt::Debug {
    delegate: Arc<UsrProviderDelegate>,
}

#[axum::async_trait]
impl <
    User: axum_login::AuthUser,
    UsrProviderDelegate: AuthUserProvider<User=User> + Send + Sync,
> AuthUserProvider for AuthUserProviderStaticTypeArcWrapper<User,UsrProviderDelegate> {
    type User = User;
    async fn get_user_by_name(&self, username: &str) -> Result<Option<Self::User>, AuthUserProviderError> {
        self.delegate.get_user_by_name(username).await
    }
    async fn get_user_by_id(&self, user_id: &<AuthUser as axum_login::AuthUser>::Id) -> Result<Option<Self::User>, AuthUserProviderError> {
        self.delegate.get_user_by_id(user_id).await
    }
}

/*
fn aa() {
    use std::sync::Arc;
    use crate::auth::InMemAuthUserProvider;

    let a1: Arc<InMemAuthUserProvider> = Arc::new(InMemAuthUserProvider::new());
    let a2: Arc<dyn AuthUserProvider<User=AuthUser>> = Arc::new(InMemAuthUserProvider::new());
    let a3: Arc<dyn AuthUserProvider<User=AuthUser>> = a1;

    let a2: Arc<dyn AuthUserProvider<User=AuthUser>> = Arc::new(AuthUserProviderStaticTypeArcWrapper { delegate: Arc::new(InMemAuthUserProvider::new()) });

    let arc1 = Arc::new(InMemAuthUserProvider::new());
    let a2: Arc<dyn AuthUserProvider<User=AuthUser>> = Arc::new(AuthUserProviderStaticTypeArcWrapper { delegate: arc1 });

    let arc1: Arc<InMemAuthUserProvider> = Arc::new(InMemAuthUserProvider::new());
    let a2: Arc<dyn AuthUserProvider<User=AuthUser>> = Arc::new(AuthUserProviderStaticTypeArcWrapper { delegate: arc1 });

    let arc1: Arc<InMemAuthUserProvider> = Arc::new(InMemAuthUserProvider::new());
    let a2: Arc<dyn AuthUserProvider<User=AuthUser>> = f2(arc1);

    let arc1: Arc<InMemAuthUserProvider> = Arc::new(InMemAuthUserProvider::new());
    let a2: Arc<dyn AuthUserProvider<User=AuthUser>> = wrap_static_arc_auth_user_provider(arc1);
}

fn f1(arc1: Arc<InMemAuthUserProvider>) -> Arc<dyn AuthUserProvider<User=AuthUser>> {
    let a2: Arc<dyn AuthUserProvider<User=AuthUser>> = Arc::new(AuthUserProviderStaticTypeArcWrapper { delegate: arc1 });
    a2
}

fn f2 <T: AuthUserProvider<User=AuthUser> + Send + Sync + 'static>
    (arc1: Arc<T>) -> Arc<dyn AuthUserProvider<User=AuthUser>> {
    let a2: Arc<dyn AuthUserProvider<User=AuthUser>> = Arc::new(AuthUserProviderStaticTypeArcWrapper { delegate: arc1 });
    a2
}
*/

pub fn wrap_static_arc_auth_user_provider<
    User: axum_login::AuthUser + Send + Sync + 'static,
    T: AuthUserProvider<User=User> + Send + Sync + 'static>
    (delegate: Arc<T>) -> Arc<dyn AuthUserProvider<User=User> + Send + Sync> {
    let casted_ptr: Arc<dyn AuthUserProvider<User=User> + Send + Sync> = Arc::new(AuthUserProviderStaticTypeArcWrapper { delegate });
    casted_ptr
}


// fn wrap_static_arc_auth_user_provide<
//     User: axum_login::AuthUser,
//     UsrProviderDelegate: AuthUserProvider<User=User> + Send + Sync,
//     >(delegate: Arc<UsrProviderDelegate>) -> Arc<dyn AuthUserProvider<User=User> /*+ Send + Sync*/> {
//     // let p: Arc<dyn AuthUserProvider<User=User> + Send + Sync> = Arc::<AuthUserProviderStaticTypeArcWrapper<User, UsrProviderDelegate>>::new(delegate);
//     let p: Arc<dyn AuthUserProvider<User=User>> = Arc::new(delegate);
//     p
// }


/*
fn aaa() {
    // core::ops::Deref
}

#[derive(Debug)]
struct AuthUserProviderWrapper<
    User: axum_login::AuthUser,
    //UsrProvider: AuthUserProvider<User=User> + Send + Sync,
    UsrProviderDelegate: core::ops::Deref<Target = dyn AuthUserProvider<User=User> + Send + Sync> + Send + Sync,
    > where UsrProviderDelegate: fmt::Debug {
    delegate: UsrProviderDelegate,
}
/*
#[axum::async_trait]
impl <
    User:axum_login::AuthUser,
    UsrProvider: AuthUserProvider<User=User> + Send + Sync,
    > AuthUserProvider for AuthUserProviderWrapper<User,UsrProvider> {
    type User = User;
    async fn get_user_by_name(&self, username: &str) -> Result<Option<Self::User>, AuthUserProviderError> {
        self.delegate.get_user_by_name(username).await
    }
    async fn get_user_by_id(&self, user_id: &<AuthUser as axum_login::AuthUser>::Id) -> Result<Option<Self::User>, AuthUserProviderError> {
        self.delegate.get_user_by_id(user_id).await
    }
}
*/
#[axum::async_trait]
impl <
    User:axum_login::AuthUser + Sync + Send,
    // UsrProvider: AuthUserProvider<User=User> + Send + Sync,
    UsrProviderDelegate: core::ops::Deref<Target = dyn AuthUserProvider<User=User> + Send + Sync> + Send + Sync,
    > AuthUserProvider for Arc<AuthUserProviderWrapper<User,/*UsrProvider,*/UsrProviderDelegate>> where UsrProviderDelegate: fmt::Debug {
    type User = User;
    async fn get_user_by_name(&self, username: &str) -> Result<Option<Self::User>, AuthUserProviderError> {
        self.delegate.get_user_by_name(username).await
    }
    async fn get_user_by_id(&self, user_id: &<AuthUser as axum_login::AuthUser>::Id) -> Result<Option<Self::User>, AuthUserProviderError> {
        self.delegate.get_user_by_id(user_id).await
    }
}

fn wrap_auth_user_provider<
    User: axum_login::AuthUser + Sync + Send,
    // UsrProvider: AuthUserProvider<User=User> + Send + Sync,
    UsrProviderDelegate: core::ops::Deref<Target = dyn AuthUserProvider<User=User> + Send + Sync> + Send + Sync,
    >(auth_user_provider: UsrProviderDelegate) ->
    // Arc<dyn AuthUserProvider<User=User> + Send + Sync>
    Arc<dyn AuthUserProvider<User=User>>
    where UsrProviderDelegate: fmt::Debug
{
    // let aa: Arc<AuthUserProviderWrapper<User, UsrProvider>> = Arc::new(AuthUserProviderWrapper::<User, UsrProvider> { delegate: auth_user_provider.clone() } );
    // let aa: Arc<dyn AuthUserProvider<User=User>> = Arc::new(AuthUserProviderWrapper::<User, UsrProvider> { delegate: auth_user_provider.clone() } );
    //let p1: Arc<UsrProvider> = Arc::clone(&auth_user_provider);
    // let aa00 = Arc::new(AuthUserProviderWrapper::<User, UsrProvider> { delegate: p1 } );
    let aa00: Arc<dyn AuthUserProvider<User=User>> = Arc::new(AuthUserProviderWrapper { delegate: auth_user_provider } );
    // let aa00 = Arc::new(AuthUserProviderWrapper { delegate: p1 } );
    // let p1: Arc<UsrProvider> = Arc::clone(&auth_user_provider);
    // let aa: Arc<dyn AuthUserProvider<User=User>> = Arc::new(AuthUserProviderWrapper::<User, UsrProvider> { delegate: p1 } );
    // aa
    //todo!()
    aa00
}
*/

#[derive(Debug, thiserror::Error)]
pub enum AuthUserProviderError {
    // 1) It is used only for updates.
    // 2) If user is not found on get operation, just Ok(None) is returned.
    #[error("UserNotFound")]
    UserNotFound,

    #[error(transparent)]
    Sqlx(sqlx::Error),

    #[error("LockedResourceError")]
    LockedResourceError,
}

impl From<sqlx::Error> for AuthUserProviderError {
    fn from(value: sqlx::Error) -> Self {
        AuthUserProviderError::Sqlx(value)
    }
}
