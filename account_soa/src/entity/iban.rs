use sqlx::{ database::HasValueRef, error::BoxDynError };
use sqlx_postgres::Postgres;
use iban::Iban;
use sqlx::database::HasArguments;
use sqlx::encode::IsNull;
//--------------------------------------------------------------------------------------------------



#[derive(educe::Educe)] #[educe(Debug)]
#[derive(derive_more::Display)]
#[display(fmt = "{}", _0)]
pub struct IbanWrapper (
    // #[educe(Debug(method(bd_dbg_fmt)))]
    pub Iban);


impl<'r> sqlx::Encode<'r, Postgres> for IbanWrapper {
    fn encode_by_ref(&self, buf: &mut <Postgres as HasArguments<'r>>::ArgumentBuffer) -> IsNull {
        <&str as sqlx::Encode<Postgres>>::encode(self.0.as_str(), buf)
    }
}

impl<'r> sqlx::Decode<'r, Postgres> for IbanWrapper {
    fn decode(value: <Postgres as HasValueRef<'r>>::ValueRef) -> Result<Self, BoxDynError> {
        // !!! to_string() returns value with spaces.
        //     We need value without spaces
        let as_str = value.as_str() ?;

        use core::str::FromStr;
        Ok(IbanWrapper(Iban::from_str(as_str) ?))
    }
}

impl sqlx::Type<Postgres> for IbanWrapper {
    fn type_info() -> <Postgres as sqlx::Database>::TypeInfo {
        <str as sqlx::Type<Postgres>>::type_info()
        // <Postgres as sqlx::Database>::TypeInfo::with_name("VARCHAR") // "IBAN")
    }
    fn compatible(ty: &<Postgres as sqlx::Database>::TypeInfo) -> bool {
        <str as sqlx::Type<Postgres>>::compatible(ty)
    }
}
