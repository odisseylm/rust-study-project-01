use anyhow::anyhow;
use sqlx_postgres::PgConnectOptions;
use crate::env::{ env_var, EnvVarOps };
//--------------------------------------------------------------------------------------------------


pub fn pg_db_connection_options() -> Result<Option<PgConnectOptions>, anyhow::Error> {

    let postgres_host = env_var("POSTGRES_HOST") ?;
    let postgres_db = env_var("POSTGRES_DB") ?;
    let postgres_user = env_var("POSTGRES_USER") ?;
    let postgres_password = env_var("POSTGRES_PASSWORD") ?;

    // if (&postgres_db, &postgres_user, &postgres_password).all_are_none() {
    if postgres_host.is_none() && postgres_db.is_none() &&
        postgres_user.is_none() && postgres_password.is_none()
    {
        return Ok(None);
    }

    let postgres_host = postgres_host.val_or_not_found_err("POSTGRES_HOST") ?;
    let postgres_db = postgres_db.val_or_not_found_err("POSTGRES_DB") ?;
    let postgres_user = postgres_user.val_or_not_found_err("POSTGRES_USER") ?;
    let postgres_password = postgres_password.val_or_not_found_err("POSTGRES_PASSWORD") ?;

    let options = PgConnectOptions::new()
        .host(postgres_host.as_str())
        .database(postgres_db.as_str())
        .application_name("rust-account-soa")
        .username(postgres_user.as_str())
        .password(postgres_password.as_str())
        ;
    Ok(Some(options))
}


pub fn pg_db_connection() -> Result<sqlx_postgres::PgPool, anyhow::Error> {
    let options = pg_db_connection_options() ?;
    let options = options.ok_or_else(||anyhow!("No Postgres DB connection options.")) ?;
    Ok(sqlx_postgres::PgPool::connect_lazy_with(options))
}



//--------------------------------------------------------------------------------------------------

#[macro_export] macro_rules! pg_column_name {
    // postgres needs lowercase (Oracle - uppercase, so on)
    ($column_name:literal) => { const_str::convert_ascii_case!(lower, $column_name) };
}


#[macro_export] macro_rules! generate_pg_delegate_type_info {
    ($Type:ty, $DelegateType:ty) => {

        #[allow(unused_imports)]
        #[allow(unused_qualifications)]
        impl sqlx::Type<sqlx_postgres::Postgres> for $Type {
            fn type_info() -> <sqlx_postgres::Postgres as sqlx::Database>::TypeInfo {
                // <Postgres as sqlx::Database>::TypeInfo::with_name("???")
                <$DelegateType as sqlx::Type<sqlx_postgres::Postgres>>::type_info()
            }
            fn compatible(ty: &<sqlx_postgres::Postgres as sqlx::Database>::TypeInfo) -> bool {
                <$DelegateType as sqlx::Type<sqlx_postgres::Postgres>>::compatible(ty)
            }
        }

    };
}


#[macro_export] macro_rules! generate_pg_ref_delegate_type_info {
    ($Type:ident, $DelegateType:ty) => { // ($Type:ty, $DelegateType:ty) => {

        #[allow(unused_imports)]
        #[allow(unused_qualifications)]
        impl<'a> sqlx::Type<sqlx_postgres::Postgres> for $Type<'a> {
            fn type_info() -> <sqlx_postgres::Postgres as sqlx::Database>::TypeInfo {
                // <Postgres as sqlx::Database>::TypeInfo::with_name("???")
                <$DelegateType as sqlx::Type<sqlx_postgres::Postgres>>::type_info()
            }
            fn compatible(ty: &<sqlx_postgres::Postgres as sqlx::Database>::TypeInfo) -> bool {
                <$DelegateType as sqlx::Type<sqlx_postgres::Postgres>>::compatible(ty)
            }
        }

    };
}


#[macro_export] macro_rules! generate_pg_delegate_encode {
    ($Type:ty, $DelegateType:ty) => {

        #[allow(unused_imports)]
        #[allow(unused_qualifications)]
        impl<'r> sqlx::Encode<'r, sqlx_postgres::Postgres> for $Type {
            fn encode_by_ref(
                &self,
                buf: &mut <sqlx_postgres::Postgres as sqlx::database::HasArguments<'r>>::ArgumentBuffer,
            ) -> sqlx::encode::IsNull {
                <& $DelegateType as sqlx::Encode<sqlx_postgres::Postgres>>::encode(&self.0, buf)
            }
        }
    }
}


#[macro_export] macro_rules! generate_pg_ref_delegate_encode {
    ($Type:ident, $DelegateType:ty) => { // ($Type:ty, $DelegateType:ty) => {

        #[allow(unused_imports)]
        #[allow(unused_qualifications)]
        impl<'r,'a> sqlx::Encode<'r, sqlx_postgres::Postgres> for $Type<'a> {
            fn encode_by_ref(
                &self,
                buf: &mut <sqlx_postgres::Postgres as sqlx::database::HasArguments<'r>>::ArgumentBuffer,
            ) -> sqlx::encode::IsNull {
                <& $DelegateType as sqlx::Encode<sqlx_postgres::Postgres>>::encode(&self.0, buf)
            }
        }
    }
}


#[macro_export] macro_rules! generate_pg_delegate_decode {
    ($Type:ident, $DelegateType:ty) => {

        #[allow(unused_imports)]
        #[allow(unused_qualifications)]
        impl<'r> sqlx::Decode<'r, sqlx_postgres::Postgres> for $Type {
            fn decode(
                value: <sqlx_postgres::Postgres as sqlx::database::HasValueRef<'r>>::ValueRef
            ) -> Result<Self, sqlx::error::BoxDynError> {
                let v = < $DelegateType > ::decode(value) ?;
                Ok( $Type (v)) // T O D O: how to use '$Type:ty' there??
            }
        }

    }
}


#[macro_export] macro_rules! generate_pg_delegate_decode_from_str {
    ($Type:ident, $DelegateType:ty) => {

        #[allow(unused_imports)]
        #[allow(unused_qualifications)]
        impl<'r> sqlx::Decode<'r, sqlx_postgres::Postgres> for $Type {
            fn decode(
                value: <sqlx_postgres::Postgres as sqlx::database::HasValueRef<'r>>::ValueRef
            ) -> Result<Self, sqlx::error::BoxDynError> {
                let as_str = <String as sqlx::Decode<'r, sqlx_postgres::Postgres>>::decode(value) ?;
                use core::str::FromStr;
                Ok( $Type (<$DelegateType>::from_str(&as_str) ?))
            }
        }

    };
}

#[macro_export] macro_rules! generate_pg_decode_from_str {
    ($Type:ty) => {

        #[allow(unused_imports)]
        #[allow(unused_qualifications)]
        impl<'r> sqlx::Decode<'r, sqlx_postgres::Postgres> for $Type {
            fn decode(
                value: <sqlx_postgres::Postgres as sqlx::database::HasValueRef<'r>>::ValueRef
            ) -> Result<Self, sqlx::error::BoxDynError> {
                let as_str: &str = <&str as sqlx::Decode<'r, sqlx_postgres::Postgres>>::decode(value) ?;
                use core::str::FromStr;
                Ok( <$Type>::from_str(as_str) ?)
            }
        }

    };
}


/*
impl<'r> sqlx::Encode<'r, Postgres> for AccountId {
    fn encode_by_ref(&self, buf: &mut <Postgres as HasArguments<'r>>::ArgumentBuffer) -> sqlx::encode::IsNull {
        <&uuid::Uuid as sqlx::Encode<Postgres>>::encode(&self.0, buf)
    }
}
impl<'r> sqlx::Decode<'r, Postgres> for AccountId {
    fn decode(value: <Postgres as HasValueRef<'r>>::ValueRef) -> Result<Self, BoxDynError> {
        let uuid = uuid::Uuid::decode(value) ?;
        Ok(AccountId(uuid))
    }
}
*/
