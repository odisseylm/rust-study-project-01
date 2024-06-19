use core::fmt;
use std::sync::Arc;

use super::super::{
    user_provider::{ AuthUserProvider, AuthUserProviderError },
    user_provider::InMemAuthUserProvider,
};


#[derive(Debug)]
struct AuthUserProviderStaticTypePtrWrapper<
    User: axum_login::AuthUser,
    UsrProviderDelegate: AuthUserProvider<User=User> + Send + Sync,
    UsrProviderDelegatePtr: core::ops::Deref<Target=UsrProviderDelegate> + Send + Sync,
> where
    UsrProviderDelegate: fmt::Debug,
    UsrProviderDelegatePtr: fmt::Debug,
{
    delegate: UsrProviderDelegatePtr,
}

#[axum::async_trait]
impl <
    User: axum_login::AuthUser,
    UsrProviderDelegate: AuthUserProvider<User=User> + Send + Sync,
    UsrProviderDelegatePtr: core::ops::Deref<Target=UsrProviderDelegate> + Send + Sync,
> AuthUserProvider for AuthUserProviderStaticTypePtrWrapper<User,UsrProviderDelegate,UsrProviderDelegatePtr>
    where
        UsrProviderDelegate: fmt::Debug,
        UsrProviderDelegatePtr: fmt::Debug,
{
    type User = User;
    //noinspection DuplicatedCode
    async fn get_user_by_principal_identity(&self, user_id: &<Self::User as axum_login::AuthUser>::Id) -> Result<Option<Self::User>, AuthUserProviderError> {
        self.delegate.get_user_by_principal_identity(user_id).await
    }
}

/// It wraps any pointer (which implements Deref) with Arc.
/// You can pass Box or Arc, but it has real sense only for Arc
/// because only with Arc (or Rc) you can reuse previous/same instance.
pub fn wrap_static_ptr_auth_user_provider<
    User: axum_login::AuthUser + 'static,
    UsrProviderDelegate: AuthUserProvider<User=User> + Send + Sync + 'static,
    UsrProviderDelegatePtr: core::ops::Deref<Target=UsrProviderDelegate> + Send + Sync + 'static,
> (delegate: UsrProviderDelegatePtr) -> Arc<dyn AuthUserProvider<User=User> + Send + Sync>
    where
        UsrProviderDelegate: fmt::Debug,
        UsrProviderDelegatePtr: fmt::Debug,
{
    let casted_ptr: Arc<dyn AuthUserProvider<User=User> + Send + Sync> = Arc::new(AuthUserProviderStaticTypePtrWrapper { delegate });
    casted_ptr
}

#[allow(dead_code)]
fn compile_test() {
    use std::sync::Arc;
    use crate::auth::examples::auth_user::AuthUserExample;


    let a1: Arc<InMemAuthUserProvider<AuthUserExample>> = Arc::new(InMemAuthUserProvider::new());
    let _a2: Arc<dyn AuthUserProvider<User=AuthUserExample>> = Arc::new(InMemAuthUserProvider::new());
    let _a3: Arc<dyn AuthUserProvider<User=AuthUserExample>> = a1;

    let arc1: Arc<InMemAuthUserProvider<AuthUserExample>> = Arc::new(InMemAuthUserProvider::new());
    let _a2: Arc<dyn AuthUserProvider<User=AuthUserExample>> = Arc::new(AuthUserProviderStaticTypePtrWrapper { delegate: arc1 });

    let arc1: Arc<InMemAuthUserProvider<AuthUserExample>> = Arc::new(InMemAuthUserProvider::new());
    let _a2: Arc<dyn AuthUserProvider<User=AuthUserExample>> = wrap_static_ptr_auth_user_provider(arc1);

    let a1: Box<InMemAuthUserProvider<AuthUserExample>> = Box::new(InMemAuthUserProvider::new());
    let _a2: Box<dyn AuthUserProvider<User=AuthUserExample>> = Box::new(InMemAuthUserProvider::new());
    let _a3: Box<dyn AuthUserProvider<User=AuthUserExample>> = a1;

    let arc1: Box<InMemAuthUserProvider<AuthUserExample>> = Box::new(InMemAuthUserProvider::new());
    let _a2: Box<dyn AuthUserProvider<User=AuthUserExample>> = Box::new(AuthUserProviderStaticTypePtrWrapper { delegate: arc1 });

    let arc1: Box<InMemAuthUserProvider<AuthUserExample>> = Box::new(InMemAuthUserProvider::new());
    let _a2: Arc<dyn AuthUserProvider<User=AuthUserExample>> = wrap_static_ptr_auth_user_provider(arc1);
}


#[cfg(test)]
mod tests {
    use super::*;
    // use super::super::{ AuthUser, AuthUserProvider };
    // use super::{ AuthUserProviderStaticTypeArcWrapper, AuthUserProviderStaticTypePtrWrapper, f2, wrap_static_arc_auth_user_provider};
    use crate::auth::examples::auth_user::AuthUserExample;

    #[derive(Debug)]
    struct AuthUserProviderStaticTypeArcWrapper<
        User: axum_login::AuthUser,
        UsrProviderDelegate: AuthUserProvider<User=User> + Send + Sync,
    > where UsrProviderDelegate: fmt::Debug {
        delegate: Arc<UsrProviderDelegate>,
    }

    //noinspection DuplicatedCode
    #[axum::async_trait]
    impl <
        User: axum_login::AuthUser,
        UsrProviderDelegate: AuthUserProvider<User=User> + Send + Sync,
    > AuthUserProvider for AuthUserProviderStaticTypeArcWrapper<User,UsrProviderDelegate> {
        type User = User;
        async fn get_user_by_principal_identity(&self, user_id: &<Self::User as axum_login::AuthUser>::Id) -> Result<Option<Self::User>, AuthUserProviderError> {
            self.delegate.get_user_by_principal_identity(user_id).await
        }
    }

    pub fn wrap_static_arc_auth_user_provider<
        User: axum_login::AuthUser + Send + Sync + 'static,
        T: AuthUserProvider<User=User> + Send + Sync + 'static>
    (delegate: Arc<T>) -> Arc<dyn AuthUserProvider<User=User> + Send + Sync> {
        let casted_ptr: Arc<dyn AuthUserProvider<User=User> + Send + Sync> = Arc::new(AuthUserProviderStaticTypeArcWrapper { delegate });
        casted_ptr
    }

    fn f2 <T: AuthUserProvider<User=AuthUserExample> + Send + Sync + 'static>
    (arc1: Arc<T>) -> Arc<dyn AuthUserProvider<User=AuthUserExample>> {
        let a2: Arc<dyn AuthUserProvider<User=AuthUserExample>> = Arc::new(AuthUserProviderStaticTypeArcWrapper { delegate: arc1 });
        a2
    }

    #[test]
    #[allow(dead_code, unused_variables)]
    fn compilation_arc_test() {
        use std::sync::Arc;

        let a1: Arc<InMemAuthUserProvider<AuthUserExample>> = Arc::new(InMemAuthUserProvider::new());
        let a2: Arc<dyn AuthUserProvider<User=AuthUserExample>> = Arc::new(InMemAuthUserProvider::new());
        let a3: Arc<dyn AuthUserProvider<User=AuthUserExample>> = a1;

        let a2: Arc<dyn AuthUserProvider<User=AuthUserExample>> = Arc::new(AuthUserProviderStaticTypeArcWrapper { delegate: Arc::new(InMemAuthUserProvider::new()) });

        let arc1 = Arc::new(InMemAuthUserProvider::new());
        let a2: Arc<dyn AuthUserProvider<User=AuthUserExample>> = Arc::new(AuthUserProviderStaticTypeArcWrapper { delegate: arc1 });

        let arc1: Arc<InMemAuthUserProvider<AuthUserExample>> = Arc::new(InMemAuthUserProvider::new());
        let a2: Arc<dyn AuthUserProvider<User=AuthUserExample>> = Arc::new(AuthUserProviderStaticTypeArcWrapper { delegate: arc1 });

        let arc1: Arc<InMemAuthUserProvider<AuthUserExample>> = Arc::new(InMemAuthUserProvider::new());
        let a2: Arc<dyn AuthUserProvider<User=AuthUserExample>> = f2(arc1);

        let arc1: Arc<InMemAuthUserProvider<AuthUserExample>> = Arc::new(InMemAuthUserProvider::new());
        let a2: Arc<dyn AuthUserProvider<User=AuthUserExample>> = wrap_static_arc_auth_user_provider(arc1);

        let arc1: Arc<InMemAuthUserProvider<AuthUserExample>> = Arc::new(InMemAuthUserProvider::new());
        let a2: Arc<dyn AuthUserProvider<User=AuthUserExample>> = Arc::new(AuthUserProviderStaticTypePtrWrapper { delegate: arc1 });

        let arc1: Arc<InMemAuthUserProvider<AuthUserExample>> = Arc::new(InMemAuthUserProvider::new());
        let a2: Arc<dyn AuthUserProvider<User=AuthUserExample>> = wrap_static_ptr_auth_user_provider(arc1);
    }

    #[test]
    #[allow(dead_code, unused_variables)]
    fn compilation_box_test() {
        use std::sync::Arc;

        let a1: Box<InMemAuthUserProvider<AuthUserExample>> = Box::new(InMemAuthUserProvider::new());
        let a2: Box<dyn AuthUserProvider<User=AuthUserExample>> = Box::new(InMemAuthUserProvider::new());
        let a3: Box<dyn AuthUserProvider<User=AuthUserExample>> = a1;

        let arc1: Box<InMemAuthUserProvider<AuthUserExample>> = Box::new(InMemAuthUserProvider::new());
        let a2: Box<dyn AuthUserProvider<User=AuthUserExample>> = Box::new(AuthUserProviderStaticTypePtrWrapper { delegate: arc1 });

        let arc1: Box<InMemAuthUserProvider<AuthUserExample>> = Box::new(InMemAuthUserProvider::new());
        let a2: Arc<dyn AuthUserProvider<User=AuthUserExample>> = wrap_static_ptr_auth_user_provider(arc1);
    }
}
