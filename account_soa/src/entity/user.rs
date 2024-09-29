use serde::{ Deserialize, Serialize };
use mvv_common::{
    generate_from_str_new_type_delegate,
    generate_pg07_delegate_decode_from_str as generate_pg_delegate_decode_from_str,
    generate_pg07_delegate_type_info as generate_pg_delegate_type_info,
    // generate_pg08_delegate_decode_from_str as generate_pg_delegate_decode_from_str,
    // generate_pg08_delegate_type_info as generate_pg_delegate_type_info,
};
use mvv_common_bank_entities::id::Id;
//--------------------------------------------------------------------------------------------------



#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
#[derive(PartialEq, derive_more::Display)]
#[display("UserId({})", _0)]
pub struct UserId( #[allow(dead_code)] Id);
type UserIdFormatError = mvv_common_bank_entities::id::parse::IdFormatError;

impl UserId {
    pub fn into_inner(self) -> Id { self.0 }
    pub fn into_inner_inner(self) -> String { self.0.into_inner() }
}

// impl DataFormatError for UserIdFormatError { }

generate_from_str_new_type_delegate! { UserId, Id, UserIdFormatError }
generate_pg_delegate_type_info! { UserId, str }
// generate_pg_encode!    { UserId, }
generate_pg_delegate_decode_from_str! { UserId, Id }



pub struct User {
    pub id: UserId, // T O D O: use new()
    pub username: String,
}
