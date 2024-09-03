use std::sync::Arc;
// use anyhow::anyhow;
// add the `r2d2` feature for diesel
use diesel::{
    r2d2::{ConnectionManager, Pool},
    PgConnection,
};

// set an alias, so we don't have to keep writing out this long type
pub type PgDbPool = Pool<ConnectionManager<PgConnection>>;

// a real-world app would have more fields in the server state like
// CORS options, environment values needed for external APIs, etc.
// pub struct ServerState {
//     pub db_pool: PgDbPool
// }


pub struct Dependencies {
    pub db_pool: Arc<PgDbPool>,
}


pub fn create_dependencies() -> anyhow::Result<Dependencies> {
    // let con = establish_pooled_connection() ?;
    let db_pool = establish_pooled_connection() ?;
    Ok(Dependencies { db_pool: Arc::new(db_pool) })
}

pub fn establish_pooled_connection()
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
