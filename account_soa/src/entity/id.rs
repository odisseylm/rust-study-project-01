use mvv_common::{
    generate_from_str_new_type_delegate,
    generate_into_inner_delegate,
    generate_pg08_delegate_decode as generate_pg_delegate_decode,
    generate_pg08_delegate_encode as generate_pg_delegate_encode,
    generate_pg08_delegate_type_info as generate_pg_delegate_type_info,
};
//--------------------------------------------------------------------------------------------------



#[derive(Debug, Clone, PartialEq, derive_more::Display)] // derive_more::FromStr)]
#[display("{}", _0)]
pub struct ClientId( #[allow(dead_code)] uuid::Uuid);

generate_into_inner_delegate!   { ClientId, uuid::Uuid }
generate_from_str_new_type_delegate! { ClientId, uuid::Uuid, mvv_common::uuid::UuidFormatError }
generate_pg_delegate_type_info! { ClientId, uuid::Uuid }
generate_pg_delegate_encode!    { ClientId, uuid::Uuid }
generate_pg_delegate_decode!    { ClientId, uuid::Uuid }



#[cfg(test)]
mod tests {
    use mvv_auth::util::test_unwrap::TestResultUnwrap;
    use super::ClientId;

    #[test]
    fn client_id_from_str() {
        ClientId::from_str("00000000-0000-0000-0000-000000000001").test_unwrap();
        ClientId::from_str("{00000000-0000-0000-0000-000000000001}").test_unwrap();
    }
}
