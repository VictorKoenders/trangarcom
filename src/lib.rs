#![recursion_limit = "1024"]

#[macro_use]
extern crate diesel;
extern crate chrono;
extern crate dotenv;
extern crate failure;
extern crate r2d2;
extern crate uuid;
#[macro_use]
extern crate serde_derive;
extern crate serde;

pub mod models;
pub mod schema;

use diesel::pg::PgConnection;
use diesel::r2d2::ConnectionManager;
use dotenv::dotenv;
use r2d2::{Error, Pool};
use std::env;

#[derive(Clone)]
pub struct DbConnection {
    pub(crate) conn: Pool<ConnectionManager<PgConnection>>,
}

pub fn establish_connection() -> Result<DbConnection, Error> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    Ok(DbConnection {
        conn: Pool::new(ConnectionManager::new(database_url))?,
    })
}
