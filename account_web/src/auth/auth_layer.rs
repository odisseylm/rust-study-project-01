use std::sync::Arc;
use mvv_auth::{
    PasswordComparator,
    AuthUserProvider,
    backend::OAuth2UserStore,
    permission::PermissionProvider,
};
use super::{
    user::{ ClientAuthUser as AuthUser, Role, RolePermissionsSet },
    backend::CompositeAuthBackend,
};
//--------------------------------------------------------------------------------------------------


pub async fn composite_auth_manager_layer <
    UsrProvider: Send + Sync + 'static
               + AuthUserProvider<User=AuthUser>
               + PermissionProvider<User=AuthUser,Permission=Role,PermissionSet=RolePermissionsSet>
               + OAuth2UserStore,
> (
    psw_comp: Arc<dyn PasswordComparator + Send + Sync>,
    user_perm_provider: Arc<UsrProvider>,
)
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

    let backend = CompositeAuthBackend::new(Arc::clone(&psw_comp), user_perm_provider) ?;
    let auth_layer: axum_login::AuthManagerLayer<CompositeAuthBackend, MemoryStore> =
        AuthManagerLayerBuilder::new(backend, session_layer).build();
    Ok(auth_layer)
}
