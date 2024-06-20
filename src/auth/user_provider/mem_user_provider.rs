use core::fmt;
use core::ops::{ Deref, DerefMut };
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;
use std::sync::Arc;

use tokio::sync::RwLock;
use crate::auth::permission::{PermissionProcessError, PermissionProvider, PermissionSet};

use super::super::{
    user_provider::{ AuthUserProvider, AuthUserProviderError },
    backend::oauth2_auth::{ OAuth2UserStore, OAuth2User },
};


struct InMemoryState <
    User: axum_login::AuthUser,
    Perm: Hash + Eq + Debug + Clone + Send + Sync,
    PermSet: PermissionSet<Permission=Perm> + Debug + Clone + Send + Sync,
> {
    // T O D O: I think we could use there Rc (instead of Arc) because it is protected by mutex... but how to say rust about it??
    // T O D O: RwLock TWICE?? It is too much... but without unsafe it is only one accessible approach.
    // users_by_id: HashMap<i64, Arc<RwLock<User>>>,
    users_by_principal_id: HashMap<User::Id, Arc<RwLock<User>>>,
    _pd: PhantomData<(Perm,PermSet)>
}
impl <
    User: axum_login::AuthUser,
    Perm: Hash + Eq + Debug + Clone + Send + Sync,
    PermSet: PermissionSet<Permission=Perm> + Debug + Clone + Send + Sync,
> InMemoryState<User,Perm,PermSet> {
    fn new() -> InMemoryState<User,Perm,PermSet> {
        InMemoryState {
            // users_by_id: HashMap::<i64, Arc<RwLock<User>>>::new(),
            users_by_principal_id: HashMap::<User::Id, Arc<RwLock<User>>>::new(),
            _pd: PhantomData,
        }
    }
    fn with_capacity(capacity: usize) -> InMemoryState<User,Perm,PermSet> {
        InMemoryState {
            // users_by_id: HashMap::<i64, Arc<RwLock<User>>>::with_capacity(capacity),
            users_by_principal_id: HashMap::<User::Id, Arc<RwLock<User>>>::with_capacity(capacity),
            _pd: PhantomData,
        }
    }
}


// #[derive(Clone, Debug)]
#[derive(Clone)]
pub struct InMemAuthUserProvider <
    User: axum_login::AuthUser,
    Perm: Hash + Eq + Debug + Clone + Send + Sync,
    PermSet: PermissionSet<Permission=Perm> + Debug + Clone + Send + Sync,
> {
    state: Arc<RwLock<InMemoryState<User,Perm,PermSet>>>,
}


impl <
    User: axum_login::AuthUser,
    Perm: Hash + Eq + Debug + Clone + Send + Sync,
    PermSet: PermissionSet<Permission=Perm> + Debug + Clone + Send + Sync,
> InMemAuthUserProvider<User,Perm,PermSet> where User::Id : Hash + Eq {
    pub fn new() -> InMemAuthUserProvider<User,Perm,PermSet> {
        InMemAuthUserProvider {
            state: Arc::new(RwLock::<InMemoryState<User,Perm,PermSet>>::new(InMemoryState::new())),
        }
    }

    pub fn with_users(users: Vec<User>) -> Result<InMemAuthUserProvider<User,Perm,PermSet>, AuthUserProviderError> {

        let mut in_memory_state = InMemoryState::with_capacity(users.len());
        for user in users {
            let user_ref = Arc::new(RwLock::new(user.clone()));

            // in_memory_state.users_by_id.insert(user.id, Arc::clone(&user_ref));
            in_memory_state.users_by_principal_id.insert(user.id(), Arc::clone(&user_ref));
        }

        Ok(InMemAuthUserProvider {
            state: Arc::new(RwLock::<InMemoryState<User,Perm,PermSet>>::new(in_memory_state)),
        })
    }
}


impl <
    User: axum_login::AuthUser,
    Perm: Hash + Eq + Debug + Clone + Send + Sync,
    PermSet: PermissionSet<Permission=Perm> + Debug + Clone + Send + Sync,
> Debug for InMemAuthUserProvider<User,Perm,PermSet> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // T O D O: how to write it with full state for async ??
        write!(f, "InMemAuthUserProvider {{ ... }}")
    }
}


async fn extract_cloned_value<T: Clone, E>(map_value: Option<&Arc<RwLock<T>>>) -> Result<Option<T>, E> {
    match map_value {
        None => Ok(None),
        Some(map_value) => {
            let v = map_value.read().await;
            Ok(Some(v.deref().clone()))
        }
    }
}
#[inline(always)]
async fn extract_cloned_user <
    User: axum_login::AuthUser,
>(map_value: Option<&Arc<RwLock<User>>>) -> Result<Option<User>, AuthUserProviderError> {
    extract_cloned_value::<User, AuthUserProviderError>(map_value).await
}

#[axum::async_trait]
impl <
    User: axum_login::AuthUser,
    Perm: Hash + Eq + Debug + Clone + Send + Sync,
    PermSet: PermissionSet<Permission=Perm> + Debug + Clone + Send + Sync,
> AuthUserProvider for InMemAuthUserProvider<User,Perm,PermSet> where User::Id : Hash + Eq {
    type User = User;
    async fn get_user_by_principal_identity(&self, user_id: &<Self::User as axum_login::AuthUser>::Id) -> Result<Option<Self::User>, AuthUserProviderError> {
        let state = self.state.read().await;
        // let username_lc = user_id.to_lowercase();
        // extract_cloned_user(state.users_by_principal_id.get(username_lc.as_str())).await

        extract_cloned_user(state.users_by_principal_id.get(&user_id)).await
    }
}

#[axum::async_trait]
impl <
    User: axum_login::AuthUser + OAuth2User,
    Perm: Hash + Eq + Debug + Clone + Send + Sync,
    PermSet: PermissionSet<Permission=Perm> + Debug + Clone + Send + Sync,
> OAuth2UserStore for InMemAuthUserProvider<User,Perm,PermSet> where User::Id : Hash + Eq {
    async fn update_user_access_token(&self, user_principal_id: User::Id, secret_token: &str) -> Result<Option<Self::User>, AuthUserProviderError> {
        let state = self.state.write().await;
        let map_value = state.users_by_principal_id.get(&user_principal_id.clone());
        match map_value {
            None => Ok(None),
            Some(map_value) => {
                let mut v = map_value.write().await;
                v.deref_mut().access_token_mut(Some(secret_token.to_string()));
                Ok(Some(v.deref().clone()))
            }
        }
    }
}


#[axum::async_trait]
impl <
    User: axum_login::AuthUser,
    Perm: Hash + Eq + Debug + Clone + Send + Sync,
    PermSet: PermissionSet<Permission=Perm> + Debug + Clone + Send + Sync,
> PermissionProvider for InMemAuthUserProvider<User,Perm,PermSet> where User::Id : Hash + Eq {
    type User = User;
    type Permission = Perm;
    type PermissionSet = PermSet;

    async fn get_user_permissions(&self, user: &Self::User) -> Result<Self::PermissionSet, PermissionProcessError> {
        todo!()
    }

    async fn get_user_permissions_by_principal_identity(&self, user_principal_id: <<Self as PermissionProvider>::User as axum_login::AuthUser>::Id)
        -> Result<Self::PermissionSet, PermissionProcessError> {
        todo!()
    }

    async fn get_group_permissions(&self, user: &Self::User) -> Result<Self::PermissionSet, PermissionProcessError> {
        todo!()
    }

    async fn get_group_permissions_by_principal_identity(&self, user_principal_id: <<Self as PermissionProvider>::User as axum_login::AuthUser>::Id)
        -> Result<Self::PermissionSet, PermissionProcessError> {
        todo!()
    }
}


#[cfg(test)]
mod tests {
    use crate::auth::permission::predefined::{Role, RolePermissionsSet};
    use super::super::super::examples::auth_user::AuthUserExample as AuthUser;
    use crate::util::{TestOptionUnwrap, TestResultUnwrap};
    use super::*;

    pub fn in_memory_test_users() -> Result<InMemAuthUserProvider<AuthUser,Role,RolePermissionsSet>, AuthUserProviderError> {
        InMemAuthUserProvider::with_users(vec!(AuthUser::new(1, "vovan", "qwerty")))
    }

    // macro_rules! aw {
    //   ($e:expr) => {
    //       tokio_test::block_on($e)
    //   };
    // }

    async fn some_async_fn_1() -> i32 {
        123
    }
    async fn some_async_fn_2() -> i32 {
        some_async_fn_1().await * 2
    }

    #[tokio::test]
    #[allow(non_snake_case)]
    async fn tests_TestAuthUserProvider() {

        let aa = async { 123 }.await;
        println!("aa: {}", aa);

        let bb = some_async_fn_2().await;
        println!("bb: {}", bb);

        let users = in_memory_test_users().test_unwrap();

        // -----------------------------------------------------------------------------------------
        // let usr_opt_res = users.get_user_by_id(&1i64).await;
        let usr_opt_res = users.get_user_by_principal_identity(&"vovan".to_string()).await;

        assert!(usr_opt_res.is_ok()); // no error
        let usr_opt = usr_opt_res.test_unwrap();
        assert!(usr_opt.is_some()); // and user exists

        let usr = usr_opt.test_unwrap();
        assert_eq!(usr.id, 1i64);
        assert_eq!(usr.username.as_str(), "vovan");
        assert_eq!(usr.password, Some("qwerty".to_string()));
        assert_eq!(usr.access_token, None);

        // -----------------------------------------------------------------------------------------
        let usr_opt_res = users.update_user_access_token("vovan".to_string(), "token1").await;
        println!("### usr_opt_res: {:?}", usr_opt_res);

        assert!(usr_opt_res.is_ok()); // no error
        let usr_opt = usr_opt_res.test_unwrap();
        assert!(usr_opt.is_some()); // and user exists

        let usr = usr_opt.test_unwrap();
        assert_eq!(usr.id, 1i64);
        assert_eq!(usr.username.as_str(), "vovan");
        assert_eq!(usr.password, Some("qwerty".to_string()));
        assert_ne!(usr.access_token, None);
        assert_eq!(usr.access_token, Some("token1".to_string()));

        // -----------------------------------------------------------------------------------------
        // let usr_opt_res = users.get_user_by_id(&1i64).await;
        let usr_opt_res = users.get_user_by_principal_identity(&"vovan".to_string()).await;

        assert!(usr_opt_res.is_ok()); // no error
        let usr_opt = usr_opt_res.test_unwrap();
        assert!(usr_opt.is_some()); // and user exists

        let usr = usr_opt.test_unwrap();
        assert_eq!(usr.id, 1i64);
        assert_eq!(usr.username.as_str(), "vovan");
        assert_eq!(usr.password, Some("qwerty".to_string()));
        assert_ne!(usr.access_token, None);
        assert_eq!(usr.access_token, Some("token1".to_string()));

        println!("Test tests_TestAuthUserProvider is completed.")
    }

    #[tokio::test]
    async fn test_6565() {
        let lock = Arc::new(RwLock::new(5));

        // many reader locks can be held at once
        {
            let r1 = lock.read().await;
            let r2 = lock.read().await;
            assert_eq!(*r1, 5);
            assert_eq!(*r2, 5);
        } // read locks are dropped at this point

        // only one write lock may be held, however
        {
            let mut w = lock.write().await;
            *w += 1;
            assert_eq!(*w, 6);
        } // write lock is dropped here
    }
}
