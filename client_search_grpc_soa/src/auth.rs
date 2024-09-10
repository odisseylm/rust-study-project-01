mod backend;
mod user;
mod user_perm_provider;
//--------------------------------------------------------------------------------------------------

pub use user::AuthUser;
pub use user::Role;
pub use user::RolePermissionsSet;

pub use backend::CompositeAuthBackend;
pub type AuthUserProvider = user_perm_provider::SqlUserProvider;