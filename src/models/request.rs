use crate::schema::request;
use chrono::NaiveDateTime;
use diesel::{ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl};
use failure::Error;
use uuid::Uuid;

#[derive(Insertable)]
#[table_name = "request"]
pub struct Request<'a> {
    pub time: NaiveDateTime,
    pub url: &'a str,
    pub headers: &'a str,
}

impl Request<'_> {
    pub fn save(&self, conn: &PgConnection) -> Result<Uuid, Error> {
        ::diesel::insert_into(request::table)
            .values(self)
            .returning(request::dsl::id)
            .get_result(conn)
            .map_err(Into::into)
    }

    pub fn set_response(
        time: f64,
        status_code: i16,
        id: &Uuid,
        conn: &PgConnection,
    ) -> Result<(), Error> {
        ::diesel::update(request::table.find(id))
            .set((
                request::dsl::response_time.eq(time),
                request::dsl::status_code.eq(status_code),
            ))
            .execute(conn)?;
        Ok(())
    }

    pub fn set_finish_time(time: f64, id: &Uuid, conn: &PgConnection) -> Result<(), Error> {
        ::diesel::update(request::table.find(id))
            .set(request::dsl::finish_time.eq(time))
            .execute(conn)?;
        Ok(())
    }
}
