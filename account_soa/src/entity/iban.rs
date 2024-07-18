use iban::Iban;
use mvv_common::{
    generate_pg_delegate_decode_from_str,
    generate_pg_delegate_encode,
    generate_pg_delegate_type_info,
};
//--------------------------------------------------------------------------------------------------



#[derive(Debug, derive_more::Display)]
#[display(fmt = "{}", _0)]
pub struct IbanWrapper (
    pub Iban);

generate_pg_delegate_type_info! { IbanWrapper, str }
generate_pg_delegate_encode!    { IbanWrapper, str }
generate_pg_delegate_decode_from_str! { IbanWrapper, Iban }
