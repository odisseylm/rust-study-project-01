use serde::{Deserialize, Serialize};
use crate::entities::id::Id;
use crate::generate_from_str_new_type_delegate;
//--------------------------------------------------------------------------------------------------



#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
#[derive(PartialEq, derive_more::Display)]
#[display(fmt = "UserId({})", _0)]
pub struct UserId( #[allow(dead_code)] Id);
type UserIdFormatError = crate::entities::id::parse::IdFormatError;

impl UserId {
    pub fn into_inner(self) -> Id { self.0 }
    pub fn into_inner_inner(self) -> String { self.0.into_inner() }
}

generate_from_str_new_type_delegate! { UserId, Id, UserIdFormatError }



pub struct User {
    pub id: UserId, // T O D O: use new()
    pub username: String,
}
