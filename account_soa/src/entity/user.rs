use serde::{ Deserialize, Serialize };
use sqlx::database::HasValueRef;
use sqlx::error::BoxDynError;
use sqlx_postgres::Postgres;
use mvv_common::entity::id::Id;
use mvv_common::generate_from_str_new_type_delegate;
//--------------------------------------------------------------------------------------------------



#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
#[derive(PartialEq, derive_more::Display)]
#[display(fmt = "UserId({})", _0)]
pub struct UserId( #[allow(dead_code)] Id);
type UserIdFormatError = mvv_common::entity::id::parse::IdFormatError;

impl UserId {
    pub fn into_inner(self) -> Id { self.0 }
    pub fn into_inner_inner(self) -> String { self.0.into_inner() }
}

generate_from_str_new_type_delegate! { UserId, Id, UserIdFormatError }


impl<'r> sqlx::Decode<'r, Postgres> for UserId {
    fn decode(value: <Postgres as HasValueRef<'r>>::ValueRef) -> Result<Self, BoxDynError> {
        let user_id = UserId::from_str(value.as_str() ?) ?;
        Ok(user_id)
    }
}
impl sqlx::Type<Postgres> for UserId {
    fn type_info() -> <Postgres as sqlx::Database>::TypeInfo {
        <Postgres as sqlx::Database>::TypeInfo::with_name("VARCHAR")
    }
}


pub struct User {
    pub id: UserId, // T O D O: use new()
    pub username: String,
}
