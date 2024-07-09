use core::fmt;
use core::fmt::Debug;
use core::hash::Hash;
use core::marker::PhantomData;
use core::ops::{ Deref, DerefMut };
use std::sync::Arc;
use std::collections::HashMap;

use tokio::sync::RwLock;

use crate::{
    user_provider::{ AuthUserProvider, AuthUserProviderError },
    backend::oauth2_auth::{ OAuth2UserStore, OAuth2User },
    permission::{ PermissionProcessError, PermissionProvider, PermissionSet },
};
//--------------------------------------------------------------------------------------------------


#[async_trait::async_trait]
pub trait UserPermissionsExtractor: Debug + Send + Sync {
    type User: axum_login::AuthUser;
    type Permission: Hash + Eq + Debug + Clone + Send + Sync;
    type PermissionSet: PermissionSet<Permission=Self::Permission> + Clone;
    fn extract_permissions_from_user(user: &Self::User) -> Self::PermissionSet;
}

struct InMemoryState <
    User: axum_login::AuthUser,
    Perm,
    PermSet: PermissionSet<Permission=Perm>,
    PermExtract: UserPermissionsExtractor,
> {
    // T O D O: I think we could use there Rc (instead of Arc) because it is protected by mutex... but how to say rust about it??
    // T O D O: RwLock TWICE?? It is too much... but without unsafe it is only one accessible approach.
    // users_by_id: HashMap<i64, Arc<RwLock<User>>>,
    users_by_principal_id: HashMap<User::Id, Arc<RwLock<User>>>,
    _pd: PhantomData<(Perm,PermSet,PermExtract)>
}
impl <
    Usr: axum_login::AuthUser,
    Perm,
    PermSet: PermissionSet<Permission=Perm>,
    PermExtract: UserPermissionsExtractor,
> InMemoryState<Usr,Perm,PermSet,PermExtract> {
    fn new() -> InMemoryState<Usr,Perm,PermSet,PermExtract> {
        InMemoryState {
            // users_by_id: HashMap::<i64, Arc<RwLock<User>>>::new(),
            users_by_principal_id: HashMap::<Usr::Id, Arc<RwLock<Usr>>>::new(),
            _pd: PhantomData,
        }
    }
    fn with_capacity(capacity: usize) -> InMemoryState<Usr,Perm,PermSet,PermExtract> {
        InMemoryState {
            // users_by_id: HashMap::<i64, Arc<RwLock<User>>>::with_capacity(capacity),
            users_by_principal_id: HashMap::<Usr::Id, Arc<RwLock<Usr>>>::with_capacity(capacity),
            _pd: PhantomData,
        }
    }
}


// #[derive(Clone, Debug)]
#[derive(Clone)]
pub struct InMemAuthUserProvider <
    User: axum_login::AuthUser,
    Perm: Hash + Eq + Debug + Clone + Send + Sync,
    PermSet: PermissionSet<Permission=Perm>,
    PermExtract: UserPermissionsExtractor + Clone,
> {
    state: Arc<RwLock<InMemoryState<User,Perm,PermSet,PermExtract>>>,
}


impl <
    Usr: axum_login::AuthUser,
    Perm: Hash + Eq + Debug + Clone + Send + Sync,
    PermSet: PermissionSet<Permission=Perm>,
    PermExtract: UserPermissionsExtractor + Clone,
> InMemAuthUserProvider<Usr,Perm,PermSet,PermExtract> where Usr::Id : Hash + Eq {
    pub fn new() -> InMemAuthUserProvider<Usr,Perm,PermSet,PermExtract> {
        InMemAuthUserProvider {
            state: Arc::new(RwLock::<InMemoryState<Usr,Perm,PermSet,PermExtract>>::new(InMemoryState::new())),
        }
    }

    pub fn with_users(users: Vec<Usr>)
        -> Result<InMemAuthUserProvider<Usr,Perm,PermSet,PermExtract>, AuthUserProviderError> {

        let mut in_memory_state = InMemoryState::with_capacity(users.len());
        for user in users {
            let user_ref = Arc::new(RwLock::new(user.clone()));

            // in_memory_state.users_by_id.insert(user.id, Arc::clone(&user_ref));
            in_memory_state.users_by_principal_id.insert(user.id(), Arc::clone(&user_ref));
        }

        Ok(InMemAuthUserProvider {
            state: Arc::new(RwLock::<InMemoryState<Usr,Perm,PermSet,PermExtract>>::new(in_memory_state)),
        })
    }
}


impl <
    Usr: axum_login::AuthUser,
    Perm: Hash + Eq + Debug + Clone + Send + Sync,
    PermSet: PermissionSet<Permission=Perm>,
    PermExtract: UserPermissionsExtractor + Clone,
> Debug for InMemAuthUserProvider<Usr,Perm,PermSet,PermExtract> {
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
    Usr: axum_login::AuthUser,
    Perm: Hash + Eq + Debug + Clone + Send + Sync,
    PermSet: PermissionSet<Permission=Perm> + Clone,
    PermExtract: UserPermissionsExtractor + Debug + Clone,
> AuthUserProvider for InMemAuthUserProvider<Usr,Perm,PermSet,PermExtract> where Usr::Id : Hash + Eq {
    type User = Usr;
    async fn get_user_by_principal_identity(&self, user_id: &<Self::User as axum_login::AuthUser>::Id) -> Result<Option<Self::User>, AuthUserProviderError> {
        let state = self.state.read().await;
        // let username_lc = user_id.to_lowercase();
        // extract_cloned_user(state.users_by_principal_id.get(username_lc.as_str())).await

        extract_cloned_user(state.users_by_principal_id.get(&user_id)).await
    }
}

#[axum::async_trait]
impl <
    Usr: axum_login::AuthUser + OAuth2User,
    Perm: Hash + Eq + Debug + Clone + Send + Sync,
    PermSet: PermissionSet<Permission=Perm> + Clone,
    PermExtract: UserPermissionsExtractor + Debug + Clone,
> OAuth2UserStore for InMemAuthUserProvider<Usr,Perm,PermSet,PermExtract> where Usr::Id : Hash + Eq {
    async fn update_user_access_token(&self, user_principal_id: Usr::Id, secret_token: &str) -> Result<Option<Self::User>, AuthUserProviderError> {
        let state = self.state.write().await;
        let map_value = state.users_by_principal_id.get(&user_principal_id);
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
    Usr: axum_login::AuthUser,
    Perm: Hash + Eq + Debug + Clone + Send + Sync,
    PermSet: PermissionSet<Permission=Perm> + Clone,
    PermExtract: UserPermissionsExtractor<User=Usr,Permission=Perm,PermissionSet=PermSet> + Debug + Clone,
> PermissionProvider for InMemAuthUserProvider<Usr,Perm,PermSet,PermExtract>
    where Usr::Id : Hash + Eq {
    type User = Usr;
    type Permission = Perm;
    type PermissionSet = PermSet;

    #[inline]
    async fn get_user_permissions(&self, user: &Self::User)
        -> Result<Self::PermissionSet, PermissionProcessError> {
        Ok(PermExtract::extract_permissions_from_user(user))
    }

    async fn get_user_permissions_by_principal_identity(&self, user_principal_id: <<Self as PermissionProvider>::User as axum_login::AuthUser>::Id)
        -> Result<Self::PermissionSet, PermissionProcessError> {
        let user_opt = self.get_user_by_principal_identity(&user_principal_id).await
            .map_err(|err|PermissionProcessError::GetUserError(anyhow::Error::new(err)))?;
        let user = user_opt.ok_or_else(||
            // in theory, it should not happen
            PermissionProcessError::NoUser(user_principal_id.to_string())) ?;
        self.get_user_permissions(&user).await
    }

    //noinspection DuplicatedCode
    async fn get_group_permissions(&self, _user: &Self::User)
        -> Result<Self::PermissionSet, PermissionProcessError> {
        Ok(PermissionSet::new())
    }

    //noinspection DuplicatedCode
    async fn get_group_permissions_by_principal_identity(&self, _user_principal_id: <<Self as PermissionProvider>::User as axum_login::AuthUser>::Id)
        -> Result<Self::PermissionSet, PermissionProcessError> {
        Ok(PermissionSet::new())
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        examples::auth_user::{ AuthUserExample, AuthUserExamplePswExtractor, },
        permission::predefined::{ Role, RolePermissionsSet, },
    };
    use crate::test::{ TestOptionUnwrap, TestResultUnwrap, };

    pub fn in_memory_test_users() -> Result<InMemAuthUserProvider<AuthUserExample,Role,RolePermissionsSet,AuthUserExamplePswExtractor>, AuthUserProviderError> {
        InMemAuthUserProvider::with_users(vec!(AuthUserExample::new(1, "in-mem-vovan", "qwerty")))
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
        let usr_opt_res = users.get_user_by_principal_identity(&"in-mem-vovan".to_test_string()).await;

        assert!(usr_opt_res.is_ok()); // no error
        let usr_opt = usr_opt_res.test_unwrap();
        assert!(usr_opt.is_some()); // and user exists

        let usr = usr_opt.test_unwrap();
        assert_eq!(usr.id, 1i64);
        assert_eq!(usr.username.as_str(), "in-mem-vovan");
        assert_eq!(usr.password, Some("qwerty".to_test_string()));
        assert_eq!(usr.access_token, None);

        // -----------------------------------------------------------------------------------------
        let usr_opt_res = users.update_user_access_token("in-mem-vovan".to_test_string(), "token1").await;
        println!("### usr_opt_res: {:?}", usr_opt_res);

        assert!(usr_opt_res.is_ok()); // no error
        let usr_opt = usr_opt_res.test_unwrap();
        assert!(usr_opt.is_some()); // and user exists

        let usr = usr_opt.test_unwrap();
        assert_eq!(usr.id, 1i64);
        assert_eq!(usr.username.as_str(), "in-mem-vovan");
        assert_eq!(usr.password, Some("qwerty".to_test_string()));
        assert_ne!(usr.access_token, None);
        assert_eq!(usr.access_token, Some("token1".to_test_string()));

        // -----------------------------------------------------------------------------------------
        // let usr_opt_res = users.get_user_by_id(&1i64).await;
        let usr_opt_res = users.get_user_by_principal_identity(&"in-mem-vovan".to_test_string()).await;

        assert!(usr_opt_res.is_ok()); // no error
        let usr_opt = usr_opt_res.test_unwrap();
        assert!(usr_opt.is_some()); // and user exists

        let usr = usr_opt.test_unwrap();
        assert_eq!(usr.id, 1i64);
        assert_eq!(usr.username.as_str(), "in-mem-vovan");
        assert_eq!(usr.password, Some("qwerty".to_test_string()));
        assert_ne!(usr.access_token, None);
        assert_eq!(usr.access_token, Some("token1".to_test_string()));

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
