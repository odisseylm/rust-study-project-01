use iban::Iban;
use mvv_common::{
    generate_from_str_new_type_delegate,
    generate_pg_delegate_decode_from_str,
    generate_pg_delegate_encode,
    generate_pg_delegate_type_info,
    generate_pg_ref_delegate_type_info,
    generate_pg_ref_delegate_encode,
};
//--------------------------------------------------------------------------------------------------



#[derive(Debug, derive_more::Display)]
#[display(fmt = "{}", _0)]
pub struct IbanWrapper (
    pub Iban);

generate_from_str_new_type_delegate!  { IbanWrapper, Iban, iban::ParseError }
generate_pg_delegate_type_info!       { IbanWrapper, str }
generate_pg_delegate_encode!          { IbanWrapper, str }
generate_pg_delegate_decode_from_str! { IbanWrapper, Iban }



#[derive(Debug, derive_more::Display)]
#[display(fmt = "{}", _0)]
pub struct IbanRefWrapper<'a> (
    pub &'a Iban);

generate_pg_ref_delegate_type_info!   { IbanRefWrapper, str }
generate_pg_ref_delegate_encode!      { IbanRefWrapper, str }
