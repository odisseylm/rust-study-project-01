// use core::time::Duration;
// use std::sync::Arc;
// use implicit_clone::ImplicitClone;
// use log::info;
// use tokio::sync::RwLock;
// use mvv_auth::{
//     AuthUserProvider, AuthUserProviderError,
//     backend::OAuth2UserStore,
//     permission::PermissionSet,
//     user_provider::InMemAuthUserProvider,
// };
// use mvv_auth::permission::{ PermissionProcessError, PermissionProvider };
// use super::user::{ AuthUser, Role, RolePermissionsSet, UserRolesExtractor };
// use mvv_common::cache::{AsyncCache, TtlMode, };
// -------------------------------------------------------------------------------------------------

use crate::auth::ClientFeatureSetSet;
use crate::auth::user::{ ClientAuthUser, ClientFeaturesExtractor, ClientType };

pub type PswComparator = mvv_auth::PlainPasswordComparator;

// pub type ClientAuthUserProvider = mvv_auth::user_provider::InMemAuthUserProvider<
//     ClientAuthUser, ClientType, ClientFeatureSetSet, ClientFeaturesExtractor >;
pub type ClientAuthUserProvider = crate::auth::sql_client_auth_provider::SqlClientAuthUserProvider;


pub fn in_mem_client_auth_user_provider()
    -> anyhow::Result<mvv_auth::user_provider::InMemAuthUserProvider<
        ClientAuthUser, ClientType, ClientFeatureSetSet, ClientFeaturesExtractor>> {
    use mvv_auth::user_provider::InMemAuthUserProvider;

    Ok(InMemAuthUserProvider::with_users(vec!(
        ClientAuthUser::test_std_client(
            // user ID: 101
            "00000000-0000-0000-0000-000000000001", "cheburan@ukr.net", "qwerty",
        ),
        ClientAuthUser::test_std_client(
            // user ID: 101
            "00000000-0000-0000-0000-000000000002", "bla-bla@bla.bla", "qwerty",
        ),
    )) ?)
}
