use actix_web::error::Result;
use actix_web::middleware::{Finished, Middleware, Response, Started};
use actix_web::{Binary, Body, HttpMessage, HttpRequest, HttpResponse};
use chrono;
use futures::future;
use prometheus::HistogramTimer;
use state::AppState;
use time;
use trangarcom;
use uuid::Uuid;

#[derive(Default)]
pub struct Logger;

struct LogData {
    id: Uuid,
    start: f64,
    timer: HistogramTimer,
}

impl Middleware<AppState> for Logger {
    fn start(&self, req: &mut HttpRequest<AppState>) -> Result<Started> {
        if req.cookie("anonymize_logging").is_some() {
            return Ok(Started::Done);
        }
        let timer = req.state().prometheus.request_timer.start_timer();

        let request = trangarcom::models::Request {
            time: chrono::Utc::now().naive_utc(),
            url: req.uri().to_string(),
            headers: format!("{:?}", req.headers()),
        };

        let id = request.save(&req.state().db)?;

        req.extensions_mut().insert(LogData {
            id,
            start: time::precise_time_s(),
            timer,
        });

        Ok(Started::Done)
    }

    fn response(
        &self,
        request: &mut HttpRequest<AppState>,
        resp: HttpResponse,
    ) -> Result<Response> {
        if let Some(data) = request.extensions().get::<LogData>() {
            let status_code = resp.status().as_u16() as i16;
            trangarcom::models::Request::set_response(
                time::precise_time_s() - data.start,
                status_code,
                &data.id,
                &request.state().db,
            )?;

            request
                .state()
                .prometheus
                .response
                .with_label_values(&["all"])
                .inc();

            request
                .state()
                .prometheus
                .response
                .with_label_values(&[&status_code.to_string()])
                .inc();
        }
        if let Some(size) = get_response_length(&resp) {
            println!("Response size: {}", size);
            request
                .state()
                .prometheus
                .response_size
                .observe(size as f64);
        }
        Ok(Response::Done(resp))
    }

    fn finish(&self, request: &mut HttpRequest<AppState>, _resp: &HttpResponse) -> Finished {
        if let Some(data) = request.extensions_mut().remove::<LogData>() {
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
            data.timer.observe_duration();
        }
        Finished::Done
    }
}

fn get_response_length(res: &HttpResponse) -> Option<usize> {
    match res.body() {
        Body::Empty => Some(0),
        Body::Binary(Binary::Bytes(b)) => Some(b.len()),
        Body::Binary(Binary::Slice(b)) => Some(b.len()),
        Body::Binary(Binary::SharedString(b)) => Some(b.len()),
        Body::Binary(Binary::ArcSharedString(b)) => Some(b.len()),
        Body::Binary(Binary::SharedVec(b)) => Some(b.len()),
        Body::Streaming(_) => None,
        Body::Actor(_) => None,
    }
}
