use mvv_auth::{
    AuthUserProviderError,
    permission::PermissionSet,
    user_provider::InMemAuthUserProvider,
};
use super::user::{ AuthUser, Role, RolePermissionsSet, UserRolesExtractor };
// -------------------------------------------------------------------------------------------------


pub type PswComparator = mvv_auth::PlainPasswordComparator;


pub fn in_memory_test_users()
    -> Result<InMemAuthUserProvider<AuthUser,Role,RolePermissionsSet,UserRolesExtractor>, AuthUserProviderError> {
    InMemAuthUserProvider::with_users(vec!(
        AuthUser::new(1, "vovan", "qwerty"),
        AuthUser::with_role(1, "vovan-read", "qwerty", Role::Read),
        AuthUser::with_role(1, "vovan-write", "qwerty", Role::Write),
        AuthUser::with_roles(1, "vovan-read-and-write", "qwerty",
            RolePermissionsSet::from_permissions([Role::Read, Role::Write])),
    ))
}
