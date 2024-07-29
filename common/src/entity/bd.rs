use core::fmt;
use bigdecimal::BigDecimal;
use sqlx::{ database::HasValueRef, error::BoxDynError };
use sqlx::database::HasArguments;
use sqlx::encode::IsNull;
use sqlx_postgres::Postgres;
use crate::{ generate_pg_delegate_type_info, generate_pg_ref_delegate_type_info };
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

#[inline]
fn big_decimal_to_sqlx_bd(bd: BigDecimal) -> sqlx::types::BigDecimal {
    let (digits, scale) = bd.into_bigint_and_exponent();
    sqlx::types::BigDecimal::new(digits, scale)
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
impl<'q> sqlx::Encode<'q, Postgres> for BigDecimalWrapper {
    fn encode_by_ref(&self, buf: &mut <Postgres as HasArguments<'q>>::ArgumentBuffer) -> IsNull {
        // let sqlx_bd_ref: &sqlx::types::BigDecimal = unsafe { core::mem::transmute(&self.0) };
        // TODO: try to avoid clone/copy/new
        let sqlx_bd = big_decimal_to_sqlx_bd(self.0.clone());
        let sqlx_bd_ref: &sqlx::types::BigDecimal = &sqlx_bd;
        <sqlx::types::BigDecimal as sqlx::Encode<'q, Postgres>>::encode_by_ref(sqlx_bd_ref, buf)
    }
    fn encode(self, buf: &mut <Postgres as HasArguments<'q>>::ArgumentBuffer) -> IsNull
        where Self: Sized {
        let sqlx_bd = big_decimal_to_sqlx_bd(self.0);
        <sqlx::types::BigDecimal as sqlx::Encode<'q, Postgres>>::encode(sqlx_bd, buf)
    }
    // fn size_hint(&self) -> usize {
    //     // T O D O: try to avoid clone/copy/new
    //     let sqlx_bd = big_decimal_to_sqlx_bd(self.0.clone());
    //     <sqlx::types::BigDecimal as sqlx::Encode<'q, Postgres>>::size_hint(&sqlx_bd)
    // }
    // fn produces(&self) -> Option<<Postgres as sqlx::Database>::TypeInfo> {
    //     // T O D O: try to avoid clone/copy/new
    //     let sqlx_bd = big_decimal_to_sqlx_bd(self.0.clone());
    //     <sqlx::types::BigDecimal as sqlx::Encode<'q, Postgres>>::produces(&sqlx_bd)
    // }
}
generate_pg_delegate_type_info! { BigDecimalWrapper, sqlx::types::BigDecimal }



#[derive(educe::Educe)] #[educe(Debug)]
#[derive(derive_more::Display)]
#[display(fmt = "{}", _0)]
pub struct BigDecimalRefWrapper<'a>(
    #[educe(Debug(method(bd_dbg_fmt)))]
    pub &'a BigDecimal);

generate_pg_ref_delegate_type_info! { BigDecimalRefWrapper, sqlx::types::BigDecimal }

impl<'q> sqlx::Encode<'q, Postgres> for BigDecimalRefWrapper<'_> {
    fn encode_by_ref(&self, buf: &mut <Postgres as HasArguments<'q>>::ArgumentBuffer) -> IsNull {
        // let sqlx_bd_ref: &sqlx::types::BigDecimal = unsafe { core::mem::transmute(self.0) };
        // TODO: try to avoid clone/copy/new
        let sqlx_bd = big_decimal_to_sqlx_bd(self.0.clone());
        let sqlx_bd_ref: &sqlx::types::BigDecimal = &sqlx_bd;
        <sqlx::types::BigDecimal as sqlx::Encode<'q, Postgres>>::encode_by_ref(sqlx_bd_ref, buf)
    }
    // fn size_hint(&self) -> usize {
    //     // T O D O: try to avoid clone/copy/new
    //     let sqlx_bd = big_decimal_to_sqlx_bd(self.0.clone());
    //     <sqlx::types::BigDecimal as sqlx::Encode<'q, Postgres>>::size_hint(&sqlx_bd)
    // }
    // fn produces(&self) -> Option<<Postgres as sqlx::Database>::TypeInfo> {
    //     // T O D O: try to avoid clone/copy/new
    //     let sqlx_bd = big_decimal_to_sqlx_bd(self.0.clone());
    //     <sqlx::types::BigDecimal as sqlx::Encode<'q, Postgres>>::produces(&sqlx_bd)
    // }
}
