use mvv_common::{
    generate_from_str_new_type_delegate,
    generate_pg_delegate_decode,
    generate_pg_delegate_encode,
    generate_pg_delegate_type_info,
};
//--------------------------------------------------------------------------------------------------



#[derive(Debug, Clone, PartialEq, derive_more::Display)] // derive_more::FromStr)]
#[display(fmt = "{}", _0)]
pub struct ClientId( #[allow(dead_code)] uuid::Uuid);
// pub type ClientIdFormatError = mvv_common::entity::id::parse::IdFormatError;
// pub type ClientIdFormatError = uuid::Error;

impl ClientId {
    pub fn into_inner(self) -> uuid::Uuid { self.0 }
}
generate_from_str_new_type_delegate! { ClientId, uuid::Uuid, parse_str, uuid::Error }
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
