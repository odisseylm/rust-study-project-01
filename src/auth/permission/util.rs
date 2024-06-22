use std::fmt::{ Debug, Display };
use axum::extract::{OriginalUri, Request};
use log::warn;
use crate::auth::backend::authz_backend::AuthorizationResult;
use crate::auth::permission::PermissionSet;


pub fn log_unauthorized_user <
    User: axum_login::AuthUser + 'static,
    Perm: Display + Debug + Clone + Send + Sync + 'static,
    PermSet: PermissionSet<Permission=Perm> + Display + Debug + Clone + Send + Sync + 'static,
> (user: &User, res: &AuthorizationResult<Perm, PermSet>) {
    match res {
        AuthorizationResult::Authorized => {}
        AuthorizationResult::NoPermission(ref no_permission) => {
            warn!("User [{}] is not authorized. No permissions [{}]", user.id(), no_permission);
        }
        AuthorizationResult::NoPermissions(ref no_permissions) => {
            warn!("User [{}] is not authorized. No permissions [{}]", user.id(), no_permissions);
        }
    };
}


pub fn log_unauthorized_access <
    User: axum_login::AuthUser + 'static,
    Perm: Display + Debug + Clone + Send + Sync + 'static,
    PermSet: PermissionSet<Permission=Perm> + Display + Debug + Clone + Send + Sync + 'static,
> (req: Request, user: &User, res: &AuthorizationResult<Perm, PermSet>) -> (Request,) {

    let url: String = req.extensions().get::<OriginalUri>()
        .map(|uri|uri.to_string())
        .unwrap_or_else(||String::new());

    match res {
        AuthorizationResult::Authorized => {}
        AuthorizationResult::NoPermission(ref no_permission) => {
            warn!("Unauthorized access attempt: user [{}] (mo permissions [{}]), resource: {}", user.id(), no_permission, url);
        }
        AuthorizationResult::NoPermissions(ref no_permissions) => {
            warn!("Unauthorized access attempt: user [{}] (mo permissions [{}]), resource: {}", user.id(), no_permissions, url);
        }
    };
    (req,)
}
