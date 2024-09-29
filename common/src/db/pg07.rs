use anyhow::anyhow;
use crate::env::{env_var_static, EnvVarOps };
use crate::net::ConnectionType;
//--------------------------------------------------------------------------------------------------


const POSTGRES_SSL_CERT_PATH: &'static str = "POSTGRES_SSL_CERT_PATH";


pub fn pg07_db_connection_options(connection_type: ConnectionType, app_name: &str)
    -> Result<Option<sqlx_postgres_07::PgConnectOptions>, anyhow::Error> {
    const POSTGRES_HOST: &'static str = "POSTGRES_HOST";
    const POSTGRES_DB: &'static str = "POSTGRES_DB";
    const POSTGRES_USER: &'static str = "POSTGRES_USER";
    const POSTGRES_PASSWORD: &'static str = "POSTGRES_PASSWORD";

    let postgres_host = env_var_static(POSTGRES_HOST) ?;
    let postgres_db = env_var_static(POSTGRES_DB) ?;
    let postgres_user = env_var_static(POSTGRES_USER) ?;
    let postgres_password = env_var_static(POSTGRES_PASSWORD) ?;
    let postgres_ssl_cert = env_var_static(POSTGRES_SSL_CERT_PATH) ?;

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

    let mut options = sqlx_postgres_07::PgConnectOptions::new()
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
            .ssl_mode(sqlx_postgres_07::PgSslMode::VerifyFull)   // requires usage ca.crt.pem
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

pub fn pg07_db_connection(app_name: &str, connection_type: ConnectionType) -> Result<sqlx_postgres_07::PgPool, anyhow::Error> {
    match connection_type {
        ConnectionType::Plain =>
            pg07_db_plain_connection(app_name),
        ConnectionType::Ssl =>
            pg07_db_ssl_connection(app_name),
        ConnectionType::Auto =>
            pg_db_auto_type_connection(app_name),
    }
}


fn pg07_db_plain_connection(app_name: &str) -> Result<sqlx_postgres_07::PgPool, anyhow::Error> {
    let options = pg07_db_connection_options(ConnectionType::Plain, app_name) ?;
    let options = options.ok_or_else(||anyhow!("No Postgres DB connection options.")) ?;
    Ok(sqlx_postgres_07::PgPool::connect_lazy_with(options))
}


fn pg07_db_ssl_connection(app_name: &str) -> Result<sqlx_postgres_07::PgPool, anyhow::Error> {
    let options = pg07_db_connection_options(ConnectionType::Ssl, app_name) ?;
    let options = options.ok_or_else(||anyhow!("No Postgres DB connection options.")) ?;
    Ok(sqlx_postgres_07::PgPool::connect_lazy_with(options))
}


fn pg_db_auto_type_connection(app_name: &str) -> Result<sqlx_postgres_07::PgPool, anyhow::Error> {
    match env_var_static(POSTGRES_SSL_CERT_PATH) ? {
        None => pg07_db_plain_connection(app_name),
        Some(_cert_path) => pg07_db_ssl_connection(app_name),
    }
}
