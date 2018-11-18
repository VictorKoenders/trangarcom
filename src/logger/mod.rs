use actix_web::error::Result;
use actix_web::middleware::{Finished, Middleware, Response, Started};
use actix_web::{Binary, Body, HttpRequest, HttpResponse};
use chrono::{TimeZone, Utc};
use futures::Future;
use rusoto_core::{HttpClient, Region};
use rusoto_credential::EnvironmentProvider;
use rusoto_dynamodb::{DynamoDb, DynamoDbClient, PutItemInput};
use state::AppState;
use std::sync::Arc;

mod data;
mod entry;

use self::data::LogData;
use self::entry::LogEntry;

lazy_static! {
    static ref LOGGER: Logger = Logger {
        dynamo_db_client: Arc::new(DynamoDbClient::new_with(
            HttpClient::new().unwrap(),
            EnvironmentProvider::default(),
            Region::EuCentral1,
        ))
    };
}

#[derive(Clone)]
pub struct Logger {
    dynamo_db_client: Arc<DynamoDbClient>,
}

impl Default for Logger {
    fn default() -> Logger {
        LOGGER.clone()
    }
}

impl Logger {
    pub fn log_entry(&self, entry: &LogEntry) {
        let item = entry.to_attribute_value_map();
        let input = PutItemInput {
            item,
            table_name: "trangar.com.requests".to_owned(),
            ..Default::default()
        };
        tokio::spawn(
            self.dynamo_db_client
                .put_item(input)
                .map(|_| ())
                .map_err(|e| println!("Error {:?}", e)),
        );
    }
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
            uri: request.uri(),
            headers: request.request().headers(),
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

        self.log_entry(&entry);

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
