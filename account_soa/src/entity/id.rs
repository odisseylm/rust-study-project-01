use sqlx::database::{ HasArguments, HasValueRef };
use sqlx::encode::IsNull;
use sqlx::error::BoxDynError;
use sqlx_postgres::Postgres;
use mvv_common::generate_from_str_new_type_delegate;
//--------------------------------------------------------------------------------------------------



#[derive(Debug, Clone, PartialEq, derive_more::Display)] // derive_more::FromStr)]
#[display(fmt = "{}", _0)]
pub struct ClientId( #[allow(dead_code)] uuid::Uuid);
// pub type ClientIdFormatError = mvv_common::entity::id::parse::IdFormatError;
// pub type ClientIdFormatError = uuid::Error;

impl ClientId {
    pub fn into_inner(self) -> uuid::Uuid { self.0 }
    // pub fn into_inner_inner(self) -> something_like_string // TODO: can we achieve this?
}
generate_from_str_new_type_delegate! { ClientId, uuid::Uuid, parse_str, uuid::Error }

impl<'r> sqlx::Encode<'r, Postgres> for ClientId {
    fn encode_by_ref(&self, buf: &mut <Postgres as HasArguments<'r>>::ArgumentBuffer) -> IsNull {
        <&uuid::Uuid as sqlx::Encode<Postgres>>::encode(&self.0, buf)
    }
}
impl<'r> sqlx::Decode<'r, Postgres> for ClientId {
    fn decode(value: <Postgres as HasValueRef<'r>>::ValueRef) -> Result<Self, BoxDynError> {
        let uuid = uuid::Uuid::decode(value) ?;
        Ok(ClientId(uuid))
    }
}
impl sqlx::Type<Postgres> for ClientId {
    fn type_info() -> <Postgres as sqlx::Database>::TypeInfo {
        <uuid::Uuid as sqlx::Type<Postgres>>::type_info()
        // <Postgres as sqlx::Database>::TypeInfo::with_name("UUID") // "CLIENT_ID")
    }
    fn compatible(ty: &<Postgres as sqlx::Database>::TypeInfo) -> bool {
        <uuid::Uuid as sqlx::Type<Postgres>>::compatible(ty)
    }
}


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
