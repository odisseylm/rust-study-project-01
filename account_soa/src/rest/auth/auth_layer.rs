use std::sync::Arc;
use mvv_auth::{ AuthUserProvider };
use mvv_auth::backend::{LoginFormAuthBackend, LoginFormAuthConfig, OAuth2UserStore};
use mvv_auth::permission::PermissionProvider;
// use mvv_auth::user_provider::InMemAuthUserProvider;

use super::user::{ AuthUser, Role, RolePermissionsSet };
use super::user_perm_provider::{ PswComparator };
use super::backend::CompositeAuthBackend;
//--------------------------------------------------------------------------------------------------


// pub async fn composite_auth_manager_layer<AccountS: AccountService>(dependencies: &Dependencies<AccountS>)
pub async fn composite_auth_manager_layer <
    UsrProvider: Send + Sync + 'static
               + AuthUserProvider<User=AuthUser>
               + PermissionProvider<User=AuthUser,Permission=Role,PermissionSet=RolePermissionsSet>
               + OAuth2UserStore,
> (user_perm_provider: Arc<UsrProvider>)
    -> Result<axum_login::AuthManagerLayer<CompositeAuthBackend, axum_login::tower_sessions::MemoryStore>, anyhow::Error> {

    use axum_login::{
        tower_sessions::{ cookie::SameSite, Expiry, MemoryStore, SessionManagerLayer },
        AuthManagerLayerBuilder,
    };
    use time::Duration;

    // This uses `tower-sessions` to establish a layer that will provide the session
    // as a request extension.
    //
    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_same_site(SameSite::Lax) // Ensure we send the cookie from the OAuth redirect.
        .with_expiry(Expiry::OnInactivity(Duration::days(1)));

    let backend = CompositeAuthBackend::new(user_perm_provider) ?;
    let auth_layer: axum_login::AuthManagerLayer<CompositeAuthBackend, MemoryStore> =
        AuthManagerLayerBuilder::new(backend, session_layer).build();
    Ok(auth_layer)
}


pub async fn login_form_auth_manager_layer <
    UsrProvider: Send + Sync + 'static
    + AuthUserProvider<User=AuthUser>
    + PermissionProvider<User=AuthUser,Permission=Role,PermissionSet=RolePermissionsSet>
    + OAuth2UserStore,
> (user_perm_provider: Arc<UsrProvider>, login_url: &'static str)
   -> Result<
       axum_login::AuthManagerLayer<LoginFormAuthBackend<AuthUser,PswComparator,RolePermissionsSet>, axum_login::tower_sessions::MemoryStore>,
       anyhow::Error
   > {

    use axum_login::{
        tower_sessions::{ cookie::SameSite, Expiry, MemoryStore, SessionManagerLayer },
        AuthManagerLayerBuilder,
    };
    use time::Duration;

    // This uses `tower-sessions` to establish a layer that will provide the session
    // as a request extension.
    //
    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_same_site(SameSite::Lax) // Ensure we send the cookie from the OAuth redirect.
        .with_expiry(Expiry::OnInactivity(Duration::days(1)));

    // This combines the session layer with our backend to establish the auth service
    // which will provide the auth session as a request extension.
    //
    let usr_provider = Arc::clone(&user_perm_provider);
    let permission_provider = Arc::clone(&usr_provider);

    let login_form_auth_backend = LoginFormAuthBackend::
            <AuthUser,PswComparator,RolePermissionsSet>::new(
        usr_provider,
        LoginFormAuthConfig { login_url, ..LoginFormAuthConfig::default() },
        permission_provider);

    let auth_layer: axum_login::AuthManagerLayer
        <LoginFormAuthBackend<AuthUser, PswComparator, RolePermissionsSet>, MemoryStore> =
        AuthManagerLayerBuilder::new(login_form_auth_backend, session_layer).build();
    Ok(auth_layer)
}


/*
pub fn users_and_perm_provider()
    -> Result<InMemAuthUserProvider<AuthUser,Role,RolePermissionsSet,UserRolesExtractor>, AuthUserProviderError> {
    super::user_perm_provider::in_memory_test_users()
}
*/
