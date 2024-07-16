use sqlx::{ database::HasValueRef, error::BoxDynError };
use sqlx_postgres::Postgres;
use iban::Iban;
//--------------------------------------------------------------------------------------------------



#[derive(educe::Educe)] #[educe(Debug)]
#[derive(derive_more::Display)]
#[display(fmt = "{}", _0)]
pub struct IbanWrapper (
    // #[educe(Debug(method(bd_dbg_fmt)))]
    pub Iban);


impl<'r> sqlx::Decode<'r, Postgres> for IbanWrapper {
    fn decode(value: <Postgres as HasValueRef<'r>>::ValueRef) -> Result<Self, BoxDynError> {
        let as_str = value.as_str() ?;
        use core::str::FromStr;
        let bd: iban::Iban = Iban::from_str(as_str)
            .map_err(|err| BoxDynError::from(err)) ?;
        Ok(IbanWrapper(bd))
    }
}
impl sqlx::Type<Postgres> for IbanWrapper {
    fn type_info() -> <Postgres as sqlx::Database>::TypeInfo {
        <Postgres as sqlx::Database>::TypeInfo::with_name("IBAN")
    }
}
