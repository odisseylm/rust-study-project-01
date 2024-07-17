use core::fmt;
use bigdecimal::BigDecimal;
use sqlx::{ database::HasValueRef, error::BoxDynError };
use sqlx_postgres::Postgres;
//--------------------------------------------------------------------------------------------------



// Default BigDecimal Debug impl shows very unfriendly info
pub fn bd_dbg_fmt(bd: &BigDecimal, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{bd} ({bd:?})")
}


// Actually sqlx::types::BigDecimal is bigdecimal::BigDecimal (version 0.3.1).
// I do not know is it ok, or I should force sqlx to use latest bigdecimal (version 04.5)?
// Currently I use such easy approach.
#[inline]
fn big_decimal_from_sqlx_bd(sqlx_bd: sqlx::types::BigDecimal) -> BigDecimal {
    let (digits, scale) = sqlx_bd.into_bigint_and_exponent();
    BigDecimal::new(digits, scale)
}


#[derive(educe::Educe)] #[educe(Debug)]
#[derive(derive_more::Display)]
#[display(fmt = "{}", _0)]
pub struct BigDecimalWrapper(
    #[educe(Debug(method(bd_dbg_fmt)))]
    pub BigDecimal);

impl<'r> sqlx::Decode<'r, Postgres> for BigDecimalWrapper {
    fn decode(value: <Postgres as HasValueRef<'r>>::ValueRef) -> Result<Self, BoxDynError> {
        let bd: sqlx::types::BigDecimal =
            <sqlx::types::BigDecimal as sqlx::Decode<'_, Postgres>>::decode(value) ?;
        Ok(BigDecimalWrapper(big_decimal_from_sqlx_bd(bd)))
    }
}
impl sqlx::Type<Postgres> for BigDecimalWrapper {
    fn type_info() -> <Postgres as sqlx::Database>::TypeInfo {
        <Postgres as sqlx::Database>::TypeInfo::with_name("NUMERIC") // "DECIMAL")
    }
}
