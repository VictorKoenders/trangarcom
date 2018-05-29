use actix_web::error::Result;
use actix_web::middleware::{Finished, Middleware, Response, Started};
use actix_web::{HttpRequest, HttpResponse};
use chrono;
use time;
use trangarcom;
use uuid::Uuid;
use state::AppState;

#[derive(Default)]
pub struct Logger;

#[derive(Debug)]
struct LogData {
    id: Uuid,
    start: f64,
}

impl Middleware<AppState> for Logger {
    fn start(&self, req: &mut HttpRequest<AppState>) -> Result<Started> {
        let request = trangarcom::models::Request {
            time: chrono::Utc::now().naive_utc(),
            url: req.uri().to_string(),
            remote_ip: req.peer_addr().unwrap().to_string(),
            headers: format!("{:?}", req.headers_mut()),
        };

        let id = request.save(&req.state().db).unwrap();
        req.extensions_mut().insert(LogData {
            id,
            start: time::precise_time_s(),
        });

        Ok(Started::Done)
    }

    fn response(&self, request: &mut HttpRequest<AppState>, resp: HttpResponse) -> Result<Response> {
        if let Some(data) = request.extensions().get::<LogData>() {
            trangarcom::models::Request::set_response_time(
                time::precise_time_s() - data.start,
                &data.id,
                &request.state().db,
            ).unwrap();
        }
        Ok(Response::Done(resp))
    }

    fn finish(&self, request: &mut HttpRequest<AppState>, _resp: &HttpResponse) -> Finished {
        if let Some(data) = request.extensions().get::<LogData>() {
            trangarcom::models::Request::set_finish_time(
                time::precise_time_s() - data.start,
                &data.id,
                &request.state().db,
            ).unwrap();
        }
        Finished::Done
    }
}
