use sqlx::database::HasValueRef;
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


impl<'r> sqlx::Decode<'r, Postgres> for ClientId {
    fn decode(value: <Postgres as HasValueRef<'r>>::ValueRef) -> Result<Self, BoxDynError> {
        let as_str = value.as_str() ?;
        let client_id: ClientId = ClientId::from_str(as_str)
            .map_err(|err| BoxDynError::from(err)) ?;
        Ok(client_id)
    }
}
impl sqlx::Type<Postgres> for ClientId {
    fn type_info() -> <Postgres as sqlx::Database>::TypeInfo {
        <Postgres as sqlx::Database>::TypeInfo::with_name("CLIENT_ID")
    }
}
