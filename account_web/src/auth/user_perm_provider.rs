use crate::auth::ClientFeatureSet;
use crate::auth::user::{ClientAuthUser, ClientFeaturesExtractor, ClientFeature};
// -------------------------------------------------------------------------------------------------



pub type PswComparator = mvv_auth::PlainPasswordComparator;

// pub type AuthUserProvider = mvv_auth::user_provider::InMemAuthUserProvider<
//     ClientAuthUser, ClientType, ClientFeatureSetSet, ClientFeaturesExtractor >;
pub type AuthUserProvider = crate::auth::sql_client_auth_provider::SqlClientAuthUserProvider;


pub fn in_mem_client_auth_user_provider()
    -> anyhow::Result<mvv_auth::user_provider::InMemAuthUserProvider<
        ClientAuthUser, ClientFeature, ClientFeatureSet, ClientFeaturesExtractor>> {
    use mvv_auth::user_provider::InMemAuthUserProvider;

    Ok(InMemAuthUserProvider::with_users(vec!(
        ClientAuthUser::test_std_client(
            "00000000-0000-0000-0000-000000000001", "cheburan@ukr.net", "qwerty",
        ),
        ClientAuthUser::test_std_client(
            "00000000-0000-0000-0000-000000000002", "bla-bla@bla.bla", "qwerty",
        ),
    )) ?)
}
