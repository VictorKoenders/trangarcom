use chrono::NaiveDateTime;
use diesel::result::Error;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use schema::request;
use uuid::Uuid;

use DbConnection;

#[derive(Insertable)]
#[table_name = "request"]
pub struct Request {
    pub time: NaiveDateTime,
    pub url: String,
    pub remote_ip: String,
    pub headers: String,
}

impl Request {
    pub fn save(&self, db: &DbConnection) -> Result<Uuid, Error> {
        let conn = db.conn.get()?;
        ::diesel::insert_into(request::table)
            .values(self)
            .returning(request::dsl::id)
            .get_result(&conn)
    }

    pub fn set_response_time(time: f64, id: &Uuid, db: &DbConnection) -> Result<(), Error> {
        let conn = db.conn.get()?;
        ::diesel::update(request::table.find(id))
            .set(request::dsl::response_time.eq(time))
            .execute(&conn)?;
        Ok(())
    }

    pub fn set_finish_time(time: f64, id: &Uuid, db: &DbConnection) -> Result<(), Error> {
        let conn = db.conn.get()?;
        ::diesel::update(request::table.find(id))
            .set(request::dsl::finish_time.eq(time))
            .execute(&conn)?;
        Ok(())
    }
}
