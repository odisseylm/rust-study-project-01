use core::fmt;
use core::ops::{ Deref, DerefMut };
use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;

use super::auth_user::AuthUser;
use super::auth_user_provider::{ AuthUserProvider, AuthUserProviderError };
use super::oauth2_auth::Oauth2UserProvider;


struct InMemoryState {
    // T O D O: I think we could use there Rc (instead of Arc) because it is protected by mutex... but how to say rust about it??
    // T O D O: RwLock TWICE?? It is too much... but without unsafe it is only one accessible approach.
    users_by_username: HashMap<String, Arc<RwLock<AuthUser>>>,
    users_by_id: HashMap<i64, Arc<RwLock<AuthUser>>>,
}
impl InMemoryState {
    fn new() -> InMemoryState {
        InMemoryState {
            users_by_username: HashMap::<String, Arc<RwLock<AuthUser>>>::new(),
            users_by_id: HashMap::<i64, Arc<RwLock<AuthUser>>>::new(),
        }
    }
    fn with_capacity(capacity: usize) -> InMemoryState {
        InMemoryState {
            users_by_username: HashMap::<String, Arc<RwLock<AuthUser>>>::with_capacity(capacity),
            users_by_id: HashMap::<i64, Arc<RwLock<AuthUser>>>::with_capacity(capacity),
        }
    }
}


// #[derive(Clone, Debug)]
#[derive(Clone)]
pub struct InMemAuthUserProvider {
    // state: Arc<Mutex<InMemoryState>>,
    state: Arc<RwLock<InMemoryState>>,
}
impl InMemAuthUserProvider {
    pub fn new() -> InMemAuthUserProvider {
        InMemAuthUserProvider {
            // state: Arc::new(Mutex::<InMemoryState>::new(InMemoryState::new())),
            state: Arc::new(RwLock::<InMemoryState>::new(InMemoryState::new())),
        }
    }

    pub async fn with_users(users: Vec<AuthUser>) -> Result<InMemAuthUserProvider, AuthUserProviderError> {
        let in_memory_state = {
            // let in_memory_state = Arc::new(Mutex::<InMemoryState>::new(InMemoryState::with_capacity(users.len())));
            let in_memory_state = Arc::new(RwLock::<InMemoryState>::new(InMemoryState::with_capacity(users.len())));
            // let mut guarded = in_memory_state.lock()
            {
                let mut guarded = in_memory_state.deref().write() // get_mut()
                    // .map_err(|_|AuthUserProviderError::LockedResourceError) ?;
                    .await;

                for user in users {
                    let user_ref = Arc::new(RwLock::new(user.clone()));

                    guarded.users_by_id.insert(user.id, Arc::clone(&user_ref));
                    guarded.users_by_username.insert(user.username.to_string(), Arc::clone(&user_ref));
                }
            //forget(guarded); // !!! 'forget' is risky function !!??!! It does NOT work!!
            }

            in_memory_state
        };

        Ok(InMemAuthUserProvider {
            state: in_memory_state,
        })
    }

    async fn test_users() -> Result<InMemAuthUserProvider, AuthUserProviderError> {
        Self::with_users(vec!(AuthUser::new(1, "vovan", "qwerty"))).await
    }
}


impl fmt::Debug for InMemAuthUserProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // T O D O: how to write it for async ??
        write!(f, "InMemAuthUserProvider {{ ... }}")

        /*
        // let state_res = self.state.lock();
        let state_res = self.state.deref().read();
        match state_res {
            Ok(ref state) => {
                let users = state.users_by_username.keys().map(|el|el.clone()).collect::<Vec<String>>().join(", ");
                write!(f, "InMemAuthUserProvider {{ {} }}", users)
            }
            Err(_) => write!(f, "InMemAuthUserProvider {{ Inaccessible content }}"),
        }
        */
    }
}

#[axum::async_trait]
impl AuthUserProvider for InMemAuthUserProvider {
    type User = AuthUser;
    async fn get_user_by_name(&self, username: &str) -> Result<Option<Self::User>, AuthUserProviderError> {
        let state_res = rw_lock_ok(self.state.read().await);
        match state_res {
            Err(_)    => Err(AuthUserProviderError::LockedResourceError),
            Ok(state) => {
                let map_value = state.users_by_username.get(username);
                match map_value {
                    None => Ok(None),
                    Some(map_value) => {
                        match rw_lock_ok(map_value.read().await) {
                            Err(_) => Err(AuthUserProviderError::LockedResourceError),
                            Ok(v)  => Ok(Some(v.deref().clone())),
                        }
                    }
                }
            }
        }
    }

    async fn get_user_by_id(&self, user_id: &<AuthUser as axum_login::AuthUser>::Id) -> Result<Option<Self::User>, AuthUserProviderError> {
        let state_res = rw_lock_ok(self.state.read().await);
        match state_res {
            Err(_)    => Err(AuthUserProviderError::LockedResourceError),
            Ok(state) => {
                // Ok(state.users_by_username.get(username).map(|usr| usr.deref().clone()))
                let map_value = state.users_by_id.get(user_id);
                match map_value {
                    None => Ok(None),
                    Some(map_value) => {
                        match rw_lock_ok(map_value.read().await) {
                            Err(_) => Err(AuthUserProviderError::LockedResourceError),
                            Ok(v)  => Ok(Some(v.deref().clone())),
                        }
                    }
                }
            }
        }
    }
}

fn rw_lock_ok<T>(t: T) -> Result<T, AuthUserProviderError> {
    Ok(t)
}

#[axum::async_trait]
impl Oauth2UserProvider for InMemAuthUserProvider {
    // type User = AuthUser;

    async fn update_user_access_token(&self, username: &str, secret_token: &str) -> Result<Option<Self::User>, AuthUserProviderError> {
        let state_res = rw_lock_ok(self.state.write().await);
        match state_res {
            Err(_)    => Err(AuthUserProviderError::LockedResourceError),
            Ok(state) => {
                // Ok(state.users_by_username.get(username).map(|usr| usr.deref().clone()))
                let map_value = state.users_by_username.get(username);
                match map_value {
                    None => Ok(None),
                    Some(map_value) => {
                        match rw_lock_ok(map_value.write().await) {
                            Err(_) => Err(AuthUserProviderError::LockedResourceError),
                            Ok(ref mut v)  => {
                                v.deref_mut().access_token(Some(secret_token.to_string()));
                                Ok(Some(v.clone()))
                            },
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::util::{TestOptionUnwrap, TestResultUnwrap};
    use super::*;

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

        let users = InMemAuthUserProvider::test_users().await.test_unwrap();

        // -----------------------------------------------------------------------------------------
        let usr_opt_res = users.get_user_by_id(&1i64).await;

        assert!(usr_opt_res.is_ok()); // no error
        let usr_opt = usr_opt_res.test_unwrap();
        assert!(usr_opt.is_some()); // and user exists

        let usr = usr_opt.test_unwrap();
        assert_eq!(usr.id, 1i64);
        assert_eq!(usr.username.as_str(), "vovan");
        assert_eq!(usr.password, Some("qwerty".to_string()));
        assert_eq!(usr.access_token, None);

        // -----------------------------------------------------------------------------------------
        let usr_opt_res = users.update_user_access_token("vovan", "token1").await;
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
        let usr_opt_res = users.get_user_by_id(&1i64).await;

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
