use core::fmt;
use bigdecimal::BigDecimal;
// Needed to avoid warnings for different versions of sqlx
use sqlx::types::BigDecimal as SqlxBigDecimal;
// use sqlx::{ database::HasValueRef }; // for sqlx 0.7
// use sqlx::database::HasArguments; // for sqlx 0.7
use sqlx::{ error::BoxDynError };
use sqlx::encode::IsNull;
use sqlx_postgres::Postgres;
use crate::{
    generate_pg08_delegate_type_info as generate_pg_delegate_type_info,
    generate_pg08_ref_delegate_type_info as generate_pg_ref_delegate_type_info,
};
//--------------------------------------------------------------------------------------------------



// Default BigDecimal Debug impl shows very unfriendly info
pub fn bd_dbg_fmt(bd: &BigDecimal, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{bd} ({bd:?})")
}


// Actually sqlx::types::BigDecimal is bigdecimal::BigDecimal (version 0.3.1) in sqlx-0.7.x.
// I do not know is it ok, or I should force sqlx to use latest bigdecimal (version 04.5)?
// Currently I use such easy approach.
#[inline]
fn big_decimal_from_sqlx_bd(sqlx_bd: SqlxBigDecimal) -> BigDecimal {
    let (digits, scale) = sqlx_bd.into_bigint_and_exponent();
    BigDecimal::new(digits, scale)
}

#[inline]
fn big_decimal_to_sqlx_bd(bd: BigDecimal) -> SqlxBigDecimal {
    let (digits, scale) = bd.into_bigint_and_exponent();
    SqlxBigDecimal::new(digits, scale)
}



#[derive(educe::Educe)] #[educe(Debug)]
#[derive(derive_more::Display)]
#[display("{}", _0)]
pub struct BigDecimalWrapper(
    #[educe(Debug(method(bd_dbg_fmt)))]
    pub BigDecimal);

impl<'r> sqlx::Decode<'r, Postgres> for BigDecimalWrapper {
    // For sqlx 0.7
    // fn decode(value: <Postgres as HasValueRef<'r>>::ValueRef) -> Result<Self, BoxDynError> {
    // For sqlx 0.8
    fn decode(value: <Postgres as sqlx::Database>::ValueRef<'r>) -> Result<Self, BoxDynError> {
        let bd: SqlxBigDecimal =
            <SqlxBigDecimal as sqlx::Decode<'_, Postgres>>::decode(value) ?;
        Ok(BigDecimalWrapper(big_decimal_from_sqlx_bd(bd)))
    }
}
//noinspection DuplicatedCode
impl<'q> sqlx::Encode<'q, Postgres> for BigDecimalWrapper {
    // For sqlx 0.7
    // fn encode(self, buf: &mut <Postgres as HasArguments<'q>>::ArgumentBuffer) -> IsNull
    // For sqlx 0.8
    fn encode(self, buf: &mut <Postgres as sqlx::Database>::ArgumentBuffer<'q>) -> Result<IsNull, BoxDynError>
        where Self: Sized {
        let sqlx_bd = big_decimal_to_sqlx_bd(self.0);
        <SqlxBigDecimal as sqlx::Encode<'q, Postgres>>::encode(sqlx_bd, buf)
    }
    // For sqlx 0.7
    // fn encode_by_ref(&self, buf: &mut <Postgres as HasArguments<'q>>::ArgumentBuffer) -> IsNull {
    // For sqlx 0.8
    fn encode_by_ref(&self, buf: &mut <Postgres as sqlx::Database>::ArgumentBuffer<'q>) -> Result<IsNull, BoxDynError> {
        // let sqlx_bd_ref: &SqlxBigDecimal = unsafe { core::mem::transmute(&self.0) };
        // TODO: try to avoid clone/copy/new
        let sqlx_bd = big_decimal_to_sqlx_bd(self.0.clone());
        let sqlx_bd_ref: &SqlxBigDecimal = &sqlx_bd;
        // For sqlx 0.7
        // <SqlxBigDecimal as sqlx::Encode<'q, Postgres>>::encode_by_ref(sqlx_bd_ref, buf)
        // For sqlx 0.8
        <SqlxBigDecimal as sqlx::Encode<'q, Postgres>>::encode_by_ref(sqlx_bd_ref, buf)
    }    // fn size_hint(&self) -> usize {
    //     // T O D O: try to avoid clone/copy/new
    //     let sqlx_bd = big_decimal_to_sqlx_bd(self.0.clone());
    //     <SqlxBigDecimal as sqlx::Encode<'q, Postgres>>::size_hint(&sqlx_bd)
    // }
    // fn produces(&self) -> Option<<Postgres as sqlx::Database>::TypeInfo> {
    //     // T O D O: try to avoid clone/copy/new
    //     let sqlx_bd = big_decimal_to_sqlx_bd(self.0.clone());
    //     <SqlxBigDecimal as sqlx::Encode<'q, Postgres>>::produces(&sqlx_bd)
    // }
}
generate_pg_delegate_type_info! { BigDecimalWrapper, SqlxBigDecimal }



#[derive(educe::Educe)] #[educe(Debug)]
#[derive(derive_more::Display)]
#[display("{}", _0)]
pub struct BigDecimalRefWrapper<'a>(
    #[educe(Debug(method(bd_dbg_fmt)))]
    pub &'a BigDecimal);

generate_pg_ref_delegate_type_info! { BigDecimalRefWrapper, SqlxBigDecimal }

//noinspection DuplicatedCode
impl<'q> sqlx::Encode<'q, Postgres> for BigDecimalRefWrapper<'_> {
    // For sqlx 0.7
    // fn encode_by_ref(&self, buf: &mut <Postgres as HasArguments<'q>>::ArgumentBuffer) -> IsNull {
    // For sqlx 0.8
    fn encode_by_ref(&self, buf: &mut <Postgres as sqlx::Database>::ArgumentBuffer<'q>) -> Result<IsNull, BoxDynError> {
        // let sqlx_bd_ref: &SqlxBigDecimal = unsafe { core::mem::transmute(self.0) };
        // TODO: try to avoid clone/copy/new
        let sqlx_bd = big_decimal_to_sqlx_bd(self.0.clone());
        let sqlx_bd_ref: &SqlxBigDecimal = &sqlx_bd;
        <SqlxBigDecimal as sqlx::Encode<'q, Postgres>>::encode_by_ref(sqlx_bd_ref, buf)
    }
    // fn size_hint(&self) -> usize {
    //     // T O D O: try to avoid clone/copy/new
    //     let sqlx_bd = big_decimal_to_sqlx_bd(self.0.clone());
    //     <SqlxBigDecimal as sqlx::Encode<'q, Postgres>>::size_hint(&sqlx_bd)
    // }
    // fn produces(&self) -> Option<<Postgres as sqlx::Database>::TypeInfo> {
    //     // T O D O: try to avoid clone/copy/new
    //     let sqlx_bd = big_decimal_to_sqlx_bd(self.0.clone());
    //     <SqlxBigDecimal as sqlx::Encode<'q, Postgres>>::produces(&sqlx_bd)
    // }
}
