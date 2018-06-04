use actix_web::error::Result;
use actix_web::middleware::{Finished, Middleware, Response, Started};
use actix_web::{HttpRequest, HttpResponse, HttpMessage};
use chrono;
use futures::future;
use state::AppState;
use time;
use trangarcom;
use uuid::Uuid;

#[derive(Default)]
pub struct Logger;

#[derive(Debug)]
struct LogData {
    id: Uuid,
    start: f64,
}

impl Middleware<AppState> for Logger {
    fn start(&self, req: &mut HttpRequest<AppState>) -> Result<Started> {
        if req.cookie("anonymize_logging").is_some() {
            return Ok(Started::Done);
        }
        use RequestIp;
        let request = trangarcom::models::Request {
            time: chrono::Utc::now().naive_utc(),
            url: req.uri().to_string(),
            remote_ip: req.get_ip(),
            headers: format!("{:?}", req.headers()),
        };

        let id = request.save(&req.state().db)?;

        req.extensions_mut().insert(LogData {
            id,
            start: time::precise_time_s(),
        });

        Ok(Started::Done)
    }

    fn response(
        &self,
        request: &mut HttpRequest<AppState>,
        resp: HttpResponse,
    ) -> Result<Response> {
        if let Some(data) = request.extensions().get::<LogData>() {
            trangarcom::models::Request::set_response_time(
                time::precise_time_s() - data.start,
                &data.id,
                &request.state().db,
            )?;
        }
        Ok(Response::Done(resp))
    }

    fn finish(&self, request: &mut HttpRequest<AppState>, _resp: &HttpResponse) -> Finished {
        if let Some(data) = request.extensions().get::<LogData>() {
            if let Err(e) = trangarcom::models::Request::set_finish_time(
                time::precise_time_s() - data.start,
                &data.id,
                &request.state().db,
            ) {
                use futures::Future;
                type FutureType =
                    Box<Future<Error = ::actix_web::error::Error, Item = ()> + 'static>;
                let future: FutureType = Box::new(future::err(e).map_err(Into::into));
                return Finished::Future(future);
            }
        }
        Finished::Done
    }
}
