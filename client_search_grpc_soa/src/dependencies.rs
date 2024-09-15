use std::sync::Arc;
// use anyhow::anyhow;
// add the `r2d2` feature for diesel
use diesel::{
    r2d2::{ConnectionManager, Pool},
    PgConnection,
};
use mvv_auth::AuthUserProvider;
use mvv_auth::permission::PermissionProvider;
use mvv_common::net::ConnectionType;
use crate::auth::{AuthUser, Role, RolePermissionsSet};

// set an alias, so we don't have to keep writing out this long type
pub type DieselPgDbPool = Pool<ConnectionManager<PgConnection>>;

// a real-world app would have more fields in the server state like
// CORS options, environment values needed for external APIs, etc.
// pub struct ServerState {
//     pub db_pool: PgDbPool
// }


#[derive(Debug, Clone)]
pub struct Dependencies {
    pub diesel_db_pool: Arc<DieselPgDbPool>,
    // pub sqlx_db_pool: Arc<sqlx_postgres::PgPool>,
    pub user_provider: Arc<dyn AuthUserProvider<User=AuthUser> + Send + Sync + 'static>,
    pub permission_provider: Arc<dyn PermissionProvider<User=AuthUser,Permission=Role,PermissionSet=RolePermissionsSet> + Send + Sync + 'static>,
}


pub fn create_dependencies() -> anyhow::Result<Dependencies> {
    let diesel_db_pool = Arc::new(create_diesel_pooled_connection() ?);
    let sqlx_db_pool = Arc::new(mvv_common::db::pg::pg_db_connection("client_search_soa", ConnectionType::Ssl) ?);
    let user_provider = Arc::new(crate::auth::AuthUserProvider::with_cache(sqlx_db_pool.clone()) ?);

    Ok(Dependencies {
        diesel_db_pool,
        //sqlx_db_pool,
        permission_provider: user_provider.clone(),
        user_provider,
    })
}

pub fn create_diesel_pooled_connection()
    -> anyhow::Result<Pool<ConnectionManager<PgConnection>>> {

    let postgres_host = std::env::var("POSTGRES_HOST") ?;
    let postgres_db = std::env::var("POSTGRES_DB") ?;
    let postgres_user = std::env::var("POSTGRES_USER") ?;
    let postgres_password = std::env::var("POSTGRES_PASSWORD") ?;

    // TODO: how to avoid putting user/psw to connection URL?
    let database_url = format!("postgres://{postgres_user}:{postgres_password}@{postgres_host}/{postgres_db}");

    // let database_url = std::env::var("DB_URL")
    //     .map_err(|_err|anyhow!("No DB_URL env var")) ?;

    let manager = ConnectionManager::<PgConnection>::new(&database_url);
    let pool = Pool::builder().build(manager)
        ?; // .map_err("Failed to create pool.");
    Ok(pool)
}

/*
pub fn establish_pooled_connection()
    -> anyhow::Result<PooledConnection<ConnectionManager<PgConnection>>> {

    // dotenv().ok();

    let database_url = std::env::var("DB_URL")
        .map_err(|err|anyhow!("No DB url")) ?;
        //.expect("DATABASE_URL must be set");

    let manager = ConnectionManager::<PgConnection>::new(&database_url);
    let pool = Pool::builder().build(manager)
        ?; // .map_err("Failed to create pool.");
    let conn = pool.clone().get() ?;
    Ok(conn)
}
*/
