use serde::{Deserialize, Serialize};
use crate::entities::id::Id;
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

use crate::generate_delegate_new_type_from_str;
generate_delegate_new_type_from_str! { UserId, Id, UserIdFormatError }

/*
#[inherent::inherent]
impl core::str::FromStr for UserId { // TODO: generate by macro
    type Err = UserIdFormatError;
    pub fn from_str(str: &str) -> Result<Self, <Self as core::str::FromStr>::Err> {
        let raw_id = Id::from_str(str) ?;
        Ok(Self(raw_id))
    }
}
*/
/*
impl core::fmt::Display for UserId { // T O D O: generate by macro
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}
*/


pub struct User {
    pub id: UserId, // TODO: use new()
    pub username: String,
}
