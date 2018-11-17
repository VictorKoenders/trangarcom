use actix_web::error::Result;
use actix_web::middleware::{Finished, Middleware, Response, Started};
use actix_web::{Binary, Body, HttpRequest, HttpResponse};
use chrono::{DateTime, TimeZone, Utc};
use prometheus::HistogramTimer;
use state::AppState;
use std::sync::{Arc, Mutex};

static LOGGER: Logger = Logger {
    logs: Arc::new(Mutex::new(Vec::new())),
};

#[derive(Clone)]
pub struct Logger {
    logs: Arc<Mutex<Vec<LogEntry>>>,
}

impl Default for Logger {
    fn default() -> Logger {
        LOGGER.clone()
    }
}

pub struct LogEntry {
    status_code: u16,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    response_size: Option<usize>,
}

struct LogData {
    start: DateTime<Utc>,
    timer: HistogramTimer,
}

impl Middleware<AppState> for Logger {
    fn start(&self, req: &HttpRequest<AppState>) -> Result<Started> {
        if req.cookie("anonymize_logging").is_some() {
            return Ok(Started::Done);
        }
        let timer = req.state().prometheus.request_timer.start_timer();

        req.extensions_mut().insert(LogData {
            start: Utc::now(),
            timer,
        });

        Ok(Started::Done)
    }

    fn response(&self, request: &HttpRequest<AppState>, resp: HttpResponse) -> Result<Response> {
        let mut entry = LogEntry {
            status_code: resp.status().as_u16(),
            start: Utc.timestamp_millis(0),
            end: Utc::now(),
            response_size: get_response_length(&resp),
        };

        if let Some(data) = request.extensions().get::<LogData>() {
            entry.start = data.start;
        }
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
            .with_label_values(&[&entry.status_code.to_string()])
            .inc();

        Ok(Response::Done(resp))
    }

    fn finish(&self, request: &HttpRequest<AppState>, _resp: &HttpResponse) -> Finished {
        if let Some(data) = request.extensions_mut().remove::<LogData>() {
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
        // Body::Binary(Binary::ArcSharedString(b)) => Some(b.len()),
        Body::Binary(Binary::SharedVec(b)) => Some(b.len()),
        Body::Streaming(_) => None,
        Body::Actor(_) => None,
    }
}

pub struct LogUploaderService {
    logger: Logger,
}

impl LogUploaderService {}
