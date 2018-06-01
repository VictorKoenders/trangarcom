#[macro_use]
extern crate diesel;
extern crate r2d2;
extern crate chrono;
extern crate dotenv;
extern crate uuid;
extern crate failure;
#[macro_use]
extern crate serde_derive;
extern crate serde;

pub mod models;
mod schema;

use diesel::pg::PgConnection;
use dotenv::dotenv;
use std::env;
use r2d2::{Pool, Error};
use diesel::r2d2::ConnectionManager;

#[derive(Clone)]
pub struct DbConnection {
    pub(crate) conn: Pool<ConnectionManager<PgConnection>>,
}

pub fn establish_connection() -> Result<DbConnection, Error> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    Ok(DbConnection {
        conn: Pool::new(
            ConnectionManager::new(database_url)
        )?
    })
}
