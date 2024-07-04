use anyhow::anyhow;
use sqlx_postgres::PgConnectOptions;
use crate::util::env::{env_var, EnvVarOps};


fn pg_db_connection_options() -> Result<Option<PgConnectOptions>, anyhow::Error> {

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

    let options = sqlx_postgres::PgConnectOptions::new()
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
