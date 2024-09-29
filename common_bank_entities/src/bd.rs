use core::fmt;
#[cfg(feature = "sqlx_07")]
use mvv_common::{
    generate_pg07_delegate_type_info,
    generate_pg07_ref_delegate_type_info,
};
#[cfg(feature = "sqlx_08")]
use mvv_common::{
    generate_pg08_delegate_type_info,
    generate_pg08_ref_delegate_type_info,
};
//--------------------------------------------------------------------------------------------------



// Default BigDecimal Debug impl shows very unfriendly info
pub fn bd_dbg_fmt(bd: &bigdecimal::BigDecimal, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{bd} ({bd:?})")
}


// Actually sqlx::types::BigDecimal is bigdecimal::BigDecimal (version 0.3.1) in sqlx-0.7.x.
// I do not know is it ok, or I should force sqlx to use latest bigdecimal (version 04.5)?
// Currently I use such easy approach.
#[cfg(feature = "sqlx_07")]
#[inline]
fn big_decimal_from_sqlx07_bd(sqlx_bd: sqlx_07::types::BigDecimal) -> bigdecimal::BigDecimal {
    let (digits, scale) = sqlx_bd.into_bigint_and_exponent();
    bigdecimal::BigDecimal::new(digits, scale)
}
#[cfg(feature = "sqlx_08")]
#[inline]
fn big_decimal_from_sqlx08_bd(sqlx_bd: sqlx_08::types::BigDecimal) -> bigdecimal::BigDecimal {
    let (digits, scale) = sqlx_bd.into_bigint_and_exponent();
    bigdecimal::BigDecimal::new(digits, scale)
}

#[cfg(feature = "sqlx_07")]
#[inline]
fn big_decimal_to_sqlx07_bd(bd: bigdecimal::BigDecimal) -> sqlx_07::types::BigDecimal {
    let (digits, scale) = bd.into_bigint_and_exponent();
    sqlx_07::types::BigDecimal::new(digits, scale)
}

#[cfg(feature = "sqlx_08")]
#[inline]
fn big_decimal_to_sqlx08_bd(bd: bigdecimal::BigDecimal) -> sqlx_08::types::BigDecimal {
    let (digits, scale) = bd.into_bigint_and_exponent();
    sqlx_08::types::BigDecimal::new(digits, scale)
}



#[derive(educe::Educe)] #[educe(Debug)]
#[derive(derive_more::Display)]
#[display("{}", _0)]
pub struct BigDecimalWrapper(
    #[educe(Debug(method(bd_dbg_fmt)))]
    pub bigdecimal::BigDecimal);

#[cfg(feature = "sqlx_07")]
impl<'r> sqlx_07::Decode<'r, sqlx_postgres_07::Postgres> for BigDecimalWrapper {
    fn decode(value: <sqlx_postgres_07::Postgres as sqlx_07::database::HasValueRef<'r>>::ValueRef)
        -> Result<Self, sqlx_07::error::BoxDynError> {
        let bd: sqlx_07::types::BigDecimal =
            <sqlx_07::types::BigDecimal as sqlx_07::Decode<'_, sqlx_postgres_07::Postgres>>::decode(value) ?;
        Ok(BigDecimalWrapper(big_decimal_from_sqlx07_bd(bd)))
    }
}
#[cfg(feature = "sqlx_08")]
impl<'r> sqlx_08::Decode<'r, sqlx_postgres_08::Postgres> for BigDecimalWrapper {
    fn decode(value: <sqlx_postgres_08::Postgres as sqlx_08::Database>::ValueRef<'r>)
        -> Result<Self, sqlx_08::error::BoxDynError> {
        let bd: sqlx_08::types::BigDecimal =
            <sqlx_08::types::BigDecimal as sqlx_08::Decode<'_, sqlx_postgres_08::Postgres>>::decode(value) ?;
        Ok(BigDecimalWrapper(big_decimal_from_sqlx08_bd(bd)))
    }
}


#[cfg(feature = "sqlx_07")]
//noinspection DuplicatedCode
impl<'q> sqlx_07::Encode<'q, sqlx_postgres_07::Postgres> for BigDecimalWrapper {
    fn encode(self, buf: &mut <sqlx_postgres_07::Postgres as sqlx_07::database::HasArguments<'q>>::ArgumentBuffer)
        -> sqlx_07::encode::IsNull where Self: Sized {
        let sqlx_bd = big_decimal_to_sqlx07_bd(self.0);
        <sqlx_07::types::BigDecimal as sqlx_07::Encode<'q, sqlx_postgres_07::Postgres>>::encode(sqlx_bd, buf)
    }
    fn encode_by_ref(&self, buf: &mut <sqlx_postgres_07::Postgres as sqlx_07::database::HasArguments<'q>>::ArgumentBuffer)
        -> sqlx_07::encode::IsNull {
        // let sqlx_bd_ref: &SqlxBigDecimal = unsafe { core::mem::transmute(&self.0) };
        // TODO: try to avoid clone/copy/new
        let sqlx_bd = big_decimal_to_sqlx07_bd(self.0.clone());
        let sqlx_bd_ref: &sqlx_07::types::BigDecimal = &sqlx_bd;
        <sqlx_07::types::BigDecimal as sqlx_07::Encode<'q, sqlx_postgres_07::Postgres>>::encode_by_ref(sqlx_bd_ref, buf)
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
#[cfg(feature = "sqlx_08")]
//noinspection DuplicatedCode
impl<'q> sqlx_08::Encode<'q, sqlx_postgres_08::Postgres> for BigDecimalWrapper {
    fn encode(self, buf: &mut <sqlx_postgres_08::Postgres as sqlx_08::Database>::ArgumentBuffer<'q>)
        -> Result<sqlx_08::encode::IsNull, sqlx_08::error::BoxDynError>
        where Self: Sized {
        let sqlx_bd = big_decimal_to_sqlx08_bd(self.0);
        <sqlx_08::types::BigDecimal as sqlx_08::Encode<'q, sqlx_postgres_08::Postgres>>::encode(sqlx_bd, buf)
    }
    fn encode_by_ref(&self, buf: &mut <sqlx_postgres_08::Postgres as sqlx_08::Database>::ArgumentBuffer<'q>)
        -> Result<sqlx_08::encode::IsNull, sqlx_08::error::BoxDynError> {
        // let sqlx_bd_ref: &SqlxBigDecimal = unsafe { core::mem::transmute(&self.0) };
        // TODO: try to avoid clone/copy/new
        let sqlx_bd = big_decimal_to_sqlx08_bd(self.0.clone());
        let sqlx_bd_ref: &sqlx_08::types::BigDecimal = &sqlx_bd;
        <sqlx_08::types::BigDecimal as sqlx_08::Encode<'q, sqlx_postgres_08::Postgres>>::encode_by_ref(sqlx_bd_ref, buf)
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

#[cfg(feature = "sqlx_07")]
generate_pg07_delegate_type_info! { BigDecimalWrapper, sqlx_07::types::BigDecimal }
#[cfg(feature = "sqlx_08")]
generate_pg08_delegate_type_info! { BigDecimalWrapper, sqlx_08::types::BigDecimal }



#[derive(educe::Educe)] #[educe(Debug)]
#[derive(derive_more::Display)]
#[display("{}", _0)]
pub struct BigDecimalRefWrapper<'a>(
    #[educe(Debug(method(bd_dbg_fmt)))]
    pub &'a bigdecimal::BigDecimal);

#[cfg(feature = "sqlx_07")]
generate_pg07_ref_delegate_type_info! { BigDecimalRefWrapper, sqlx_07::types::BigDecimal }
#[cfg(feature = "sqlx_08")]
generate_pg08_ref_delegate_type_info! { BigDecimalRefWrapper, sqlx_08::types::BigDecimal }

#[cfg(feature = "sqlx_07")]
//noinspection DuplicatedCode
impl<'q> sqlx_07::Encode<'q, sqlx_postgres_07::Postgres> for BigDecimalRefWrapper<'_> {
    fn encode_by_ref(&self, buf: &mut <sqlx_postgres_07::Postgres as sqlx_07::database::HasArguments<'q>>::ArgumentBuffer)
        -> sqlx_07::encode::IsNull {
        // let sqlx_bd_ref: &SqlxBigDecimal = unsafe { core::mem::transmute(self.0) };
        // TODO: try to avoid clone/copy/new
        let sqlx_bd = big_decimal_to_sqlx07_bd(self.0.clone());
        let sqlx_bd_ref: &sqlx_07::types::BigDecimal = &sqlx_bd;
        <sqlx_07::types::BigDecimal as sqlx_07::Encode<'q, sqlx_postgres_07::Postgres>>::encode_by_ref(sqlx_bd_ref, buf)
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

#[cfg(feature = "sqlx_08")]
//noinspection DuplicatedCode
impl<'q> sqlx_08::Encode<'q, sqlx_postgres_08::Postgres> for BigDecimalRefWrapper<'_> {
    fn encode_by_ref(&self, buf: &mut <sqlx_postgres_08::Postgres as sqlx_08::Database>::ArgumentBuffer<'q>)
        -> Result<sqlx_08::encode::IsNull, sqlx_08::error::BoxDynError> {
        // let sqlx_bd_ref: &SqlxBigDecimal = unsafe { core::mem::transmute(self.0) };
        // TODO: try to avoid clone/copy/new
        let sqlx_bd = big_decimal_to_sqlx08_bd(self.0.clone());
        let sqlx_bd_ref: &sqlx_08::types::BigDecimal = &sqlx_bd;
        <sqlx_08::types::BigDecimal as sqlx_08::Encode<'q, sqlx_postgres_08::Postgres>>::encode_by_ref(sqlx_bd_ref, buf)
    }
}
