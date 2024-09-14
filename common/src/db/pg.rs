use anyhow::anyhow;
use sqlx_postgres::{PgConnectOptions, PgSslMode};
use crate::env::{ env_var, EnvVarOps };
use crate::net::ConnectionType;
//--------------------------------------------------------------------------------------------------


const POSTGRES_SSL_CERT_PATH: &'static str = "POSTGRES_SSL_CERT_PATH";


pub fn pg_db_connection_options(connection_type: ConnectionType, app_name: &str) -> Result<Option<PgConnectOptions>, anyhow::Error> {
    const POSTGRES_HOST: &'static str = "POSTGRES_HOST";
    const POSTGRES_DB: &'static str = "POSTGRES_DB";
    const POSTGRES_USER: &'static str = "POSTGRES_USER";
    const POSTGRES_PASSWORD: &'static str = "POSTGRES_PASSWORD";

    let postgres_host = env_var(POSTGRES_HOST) ?;
    let postgres_db = env_var(POSTGRES_DB) ?;
    let postgres_user = env_var(POSTGRES_USER) ?;
    let postgres_password = env_var(POSTGRES_PASSWORD) ?;
    let postgres_ssl_cert = env_var(POSTGRES_SSL_CERT_PATH) ?;

    // if (&postgres_db, &postgres_user, &postgres_password).all_are_none() {
    if postgres_host.is_none() && postgres_db.is_none() &&
        postgres_user.is_none() && postgres_password.is_none()
    {
        return Ok(None);
    }

    let postgres_host = postgres_host.val_or_not_found_err(POSTGRES_HOST) ?;
    let postgres_db = postgres_db.val_or_not_found_err(POSTGRES_DB) ?;
    let postgres_user = postgres_user.val_or_not_found_err(POSTGRES_USER) ?;
    let postgres_password = postgres_password.val_or_not_found_err(POSTGRES_PASSWORD) ?;

    let mut options = PgConnectOptions::new()
        .host(postgres_host.as_str())
        .database(postgres_db.as_str())
        .application_name(app_name)
        .username(postgres_user.as_str())
        .password(postgres_password.as_str())
        ;

    if let ConnectionType::Ssl = connection_type {
        let postgres_ssl_cert = postgres_ssl_cert
            .and_then(|s| if s.is_empty() { None } else { Some(s) } )
            .val_or_not_found_err(POSTGRES_SSL_CERT_PATH) ?;
        options = options
            // .ssl_mode(PgSslMode::Require)   // database.crt.pem can be used (or ca.crt.pem)
            //.ssl_mode(PgSslMode::VerifyCa)     // requires usage ca.crt.pem
            .ssl_mode(PgSslMode::VerifyFull)   // requires usage ca.crt.pem
            // ? Why not 'server' cert ?
            //
            // In case of PgSslMode::Require database.crt.pem can be used.
            // In case of PgSslMode::VerifyXXX ca.crt.pem must be used.
            .ssl_root_cert(postgres_ssl_cert)
            //.ssl_root_cert_from_pem(std::fs::read_to_string(postgres_ssl_cert) ?.into_bytes())
            ;
    }

    Ok(Some(options))
}

pub fn pg_db_connection(app_name: &str, connection_type: ConnectionType) -> Result<sqlx_postgres::PgPool, anyhow::Error> {
    match connection_type {
        ConnectionType::Plain =>
            pg_db_plain_connection(app_name),
        ConnectionType::Ssl =>
            pg_db_ssl_connection(app_name),
        ConnectionType::Auto =>
            pg_db_auto_type_connection(app_name),
    }
}


fn pg_db_plain_connection(app_name: &str) -> Result<sqlx_postgres::PgPool, anyhow::Error> {
    let options = pg_db_connection_options(ConnectionType::Plain, app_name) ?;
    let options = options.ok_or_else(||anyhow!("No Postgres DB connection options.")) ?;
    Ok(sqlx_postgres::PgPool::connect_lazy_with(options))
}


fn pg_db_ssl_connection(app_name: &str) -> Result<sqlx_postgres::PgPool, anyhow::Error> {
    let options = pg_db_connection_options(ConnectionType::Ssl, app_name) ?;
    let options = options.ok_or_else(||anyhow!("No Postgres DB connection options.")) ?;
    Ok(sqlx_postgres::PgPool::connect_lazy_with(options))
}


fn pg_db_auto_type_connection(app_name: &str) -> Result<sqlx_postgres::PgPool, anyhow::Error> {
    match env_var(POSTGRES_SSL_CERT_PATH) ? {
        None => pg_db_plain_connection(app_name),
        Some(_cert_path) => pg_db_ssl_connection(app_name),
    }
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
                <$DelegateType as sqlx::Type<sqlx_postgres::Postgres>>::type_info()
            }
            fn compatible(ty: &<sqlx_postgres::Postgres as sqlx::Database>::TypeInfo) -> bool {
                <$DelegateType as sqlx::Type<sqlx_postgres::Postgres> >::compatible(ty)
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
                <$DelegateType as sqlx::Type<sqlx_postgres::Postgres>>::type_info()
            }
            fn compatible(ty: &<sqlx_postgres::Postgres as sqlx::Database>::TypeInfo) -> bool {
                <$DelegateType as sqlx::Type<sqlx_postgres::Postgres> >::compatible(ty)
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
                // for sql 0.7 TODO: how to have build for different versions
                // buf: &mut <sqlx_postgres::Postgres as sqlx::database::HasArguments<'r>>::ArgumentBuffer,
                // for sql 0.8
                buf: &mut <sqlx_postgres::Postgres as sqlx::Database>::ArgumentBuffer<'r>,
            // ) -> sqlx::encode::IsNull { // for sql 0.7
            ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
                <& $DelegateType as sqlx::Encode<sqlx_postgres::Postgres> >::encode(&self.0, buf)
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
                // for sqlx 0.7
                // buf: &mut <sqlx_postgres::Postgres as sqlx::database::HasArguments<'r>>::ArgumentBuffer,
                // for sqlx 0.8
                buf: &mut <sqlx_postgres::Postgres as sqlx::Database>::ArgumentBuffer<'r>,
            // ) -> sqlx::encode::IsNull { // for sqlx 0.7
            // for sqlx 0.8
            ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
                <& $DelegateType as sqlx::Encode<sqlx_postgres::Postgres> >::encode(&self.0, buf)
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
                // value: <sqlx_postgres::Postgres as sqlx::database::HasValueRef<'r>>::ValueRef
                value: <sqlx_postgres::Postgres as sqlx::Database>::ValueRef<'r>
            ) -> Result<Self, sqlx::error::BoxDynError> {
                let v = < $DelegateType as sqlx::Decode<'r, sqlx_postgres::Postgres> > ::decode(value) ?;
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
                // for sqlx 0.7
                // value: <sqlx_postgres::Postgres as sqlx::database::HasValueRef<'r>>::ValueRef
                // for sqlx 0.8
                value: <sqlx_postgres::Postgres as sqlx::Database>::ValueRef<'r>
            ) -> Result<Self, sqlx::error::BoxDynError> {
                let as_str = <String as sqlx::Decode<'r, sqlx_postgres::Postgres>>::decode(value) ?;
                Ok( $Type (<$DelegateType as core::str::FromStr>::from_str(&as_str) ?))
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
                // value: <sqlx_postgres::Postgres as sqlx::database::HasValueRef<'r>>::ValueRef
                value: <sqlx_postgres::Postgres as sqlx::Database>::ValueRef<'r>
            ) -> Result<Self, sqlx::error::BoxDynError> {
                let as_str: &str = <&str as sqlx::Decode<'r, sqlx_postgres::Postgres>>::decode(value) ?;
                Ok( <$Type as core::str::FromStr>::from_str(as_str) ?)
            }
        }

    };
}


#[macro_export] macro_rules! generate_pg_encode_from_as_str {
    ($Type:ident) => { // ($Type:ty) => {

        #[allow(unused_imports)]
        #[allow(unused_qualifications)]
        // impl<'r,'a> sqlx::Encode<'r, sqlx_postgres::Postgres> for $Type {
        impl<'q> sqlx::Encode<'q, sqlx_postgres::Postgres> for $Type {
            // For sqlx 0.7
            // fn encode_by_ref(&self, buf: &mut <Postgres as sqlx::database::HasArguments<'q>>::ArgumentBuffer) -> IsNull {
            // For sqlx 0.8
            fn encode_by_ref(&self, buf: &mut <sqlx_postgres::Postgres as sqlx::Database>::ArgumentBuffer<'q>) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
                let str: &str = self.as_str();
                <&str as sqlx::Encode<sqlx_postgres::Postgres>>::encode(str, buf)
            }
        }
    }
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
