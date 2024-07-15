use core::fmt::Display;
use axum::extract::Request;
use log::warn;

use crate::backend::authz_backend::AuthorizationResult;
use crate::http::req_original_uri_or_empty;
use crate::permission::PermissionSet;


pub fn log_unauthorized_user <
    User: axum_login::AuthUser + 'static,
    PermSet: PermissionSet + Display + Clone + 'static,
> (user: &User, res: &AuthorizationResult<PermSet>)
    where <PermSet as PermissionSet>::Permission: Display
{
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
    PermSet: PermissionSet + Display + Clone + 'static,
> (req: Request, user: &User, res: &AuthorizationResult<PermSet>)
    -> (Request,)
    where <PermSet as PermissionSet>::Permission: Display
{
    let url: String = req_original_uri_or_empty(&req);

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
