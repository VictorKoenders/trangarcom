use crate::rocket_utils::Database;
use chrono::Utc;
use rocket::{
    fairing::{Fairing, Info, Kind},
    Data, Request, Response,
};
use std::fmt::Write;
use uuid::Uuid;

pub struct Logger;

#[derive(Default)]
struct LogState {
    id: Uuid,
    start: f64,
}

impl Fairing for Logger {
    fn info(&self) -> Info {
        Info {
            name: "Request Logger",
            kind: Kind::Request | Kind::Response,
        }
    }

    fn on_request(&self, request: &mut Request, _: &Data) {
        let database: Database = request.guard().expect("Could not get database instance");
        let mut headers = request
            .headers()
            .iter()
            .fold(String::new(), |mut acc, header| {
                let c = if acc.is_empty() { "{ " } else { ", " };
                write!(&mut acc, "{}", c).expect("Could not write");
                write!(&mut acc, "{}: {}", header.name(), header.value).expect("Could not write");
                acc
            });
        headers += " }";

        let db_request = trangarcom::models::Request {
            headers: &headers,
            time: Utc::now().naive_utc(),
            url: request.uri().path(),
        };

        let id = db_request.save(&database).expect("Could not log request");

        request.local_cache(|| LogState {
            id,
            start: time::precise_time_s(),
        });
    }

    fn on_response(&self, request: &Request, response: &mut Response) {
        let database: Database = request.guard().expect("Could not get database instance");
        let state: &LogState = request.local_cache(LogState::default);
        let status = response.status();

        trangarcom::models::Request::set_response(
            time::precise_time_s() - state.start,
            status.code as i16,
            &state.id,
            &database,
        )
        .expect("Could not update database");
    }
}
