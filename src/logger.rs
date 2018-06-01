use actix_web::error::{ResponseError, Result};
use actix_web::middleware::{Finished, Middleware, Response, Started};
use actix_web::{HttpRequest, HttpResponse};
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

pub struct NoRemoteAddrError;

impl ::std::error::Error for NoRemoteAddrError {
    fn description(&self) -> &'static str {
        "No remote address set"
    }
}

impl ::std::fmt::Display for NoRemoteAddrError {
    fn fmt(&self, fmt: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        use std::error::Error;
        write!(fmt, "{}", self.description())
    }
}

impl ::std::fmt::Debug for NoRemoteAddrError {
    fn fmt(&self, fmt: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        use std::error::Error;
        write!(fmt, "{}", self.description())
    }
}

impl ResponseError for NoRemoteAddrError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::BadRequest().finish()
    }
}

impl Middleware<AppState> for Logger {
    fn start(&self, req: &mut HttpRequest<AppState>) -> Result<Started> {
        let ip = match req.peer_addr() {
            Some(a) => a.to_string(),
            None => {
                return Err(NoRemoteAddrError.into());
            }
        };
        let request = trangarcom::models::Request {
            time: chrono::Utc::now().naive_utc(),
            url: req.uri().to_string(),
            remote_ip: ip,
            headers: format!("{:?}", req.headers_mut()),
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
                type FutureType = Box<
                    Future<Error = ::actix_web::error::Error, Item = ()>
                    + 'static
                >;
                let future: FutureType = Box::new(future::err(e).map_err(Into::into));
                return Finished::Future(future);
            }
        }
        Finished::Done
    }
}
