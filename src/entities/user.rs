use serde::{Deserialize, Serialize};
use crate::entities::id::Id;
use crate::generate_delegate_new_type_from_str;
//--------------------------------------------------------------------------------------------------



#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
#[derive(PartialEq, derive_more::Display)]
#[display(fmt = "UserId({})", "0")]
pub struct UserId( #[allow(dead_code)] Id);
type UserIdFormatError = crate::entities::id::parse::IdFormatError;

impl UserId {
    pub fn move_out(self) -> Id { self.0 }
    pub fn move_string_out(self) -> String { self.0.into_inner() }
}

generate_delegate_new_type_from_str! { UserId, Id, UserIdFormatError }



pub struct User {
    pub id: UserId, // TODO: use new()
    pub username: String,
}
