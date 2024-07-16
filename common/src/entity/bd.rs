use core::fmt;
use bigdecimal::BigDecimal;
use sqlx::{ database::HasValueRef, error::BoxDynError };
use sqlx_postgres::Postgres;
//--------------------------------------------------------------------------------------------------



// Default BigDecimal Debug impl shows very unfriendly info
pub fn bd_dbg_fmt(bd: &BigDecimal, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{bd} ({bd:?})")
}


#[derive(educe::Educe)] #[educe(Debug)]
#[derive(derive_more::Display)]
#[display(fmt = "{}", _0)]
pub struct BigDecimalWrapper(
    #[educe(Debug(method(bd_dbg_fmt)))]
    pub BigDecimal);


impl<'r> sqlx::Decode<'r, Postgres> for BigDecimalWrapper {
    fn decode(value: <Postgres as HasValueRef<'r>>::ValueRef) -> Result<Self, BoxDynError> {
        let as_str = value.as_str() ?;
        use core::str::FromStr;
        let bd: BigDecimal = BigDecimal::from_str(as_str)
            .map_err(|err| BoxDynError::from(err)) ?;
        Ok(BigDecimalWrapper(bd))
    }
}
impl sqlx::Type<Postgres> for BigDecimalWrapper {
    fn type_info() -> <Postgres as sqlx::Database>::TypeInfo {
        <Postgres as sqlx::Database>::TypeInfo::with_name("DECIMAL")
    }
}
