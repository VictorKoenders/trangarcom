use actix_web::error::Result;
use actix_web::middleware::{Finished, Middleware, Response, Started};
use actix_web::{Binary, Body, HttpRequest, HttpResponse};
use chrono::{DateTime, TimeZone, Utc};
use futures::Future;
use prometheus::HistogramTimer;
use rusoto_core::{HttpClient, Region};
use rusoto_credential::EnvironmentProvider;
use rusoto_dynamodb::{AttributeValue, DynamoDb, DynamoDbClient, PutItemInput};
use state::AppState;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

lazy_static! {
    static ref LOGGER: Logger = Logger {
        logs: Arc::new(Mutex::new(Vec::new())),
        dynamo_db_client: Arc::new(DynamoDbClient::new_with(
            HttpClient::new().unwrap(),
            EnvironmentProvider::default(),
            Region::EuCentral1,
        ))
    };
}

#[derive(Clone)]
pub struct Logger {
    logs: Arc<Mutex<Vec<LogEntry>>>,
    dynamo_db_client: Arc<DynamoDbClient>,
}

impl Logger {
    pub fn log_entry(&self, entry: LogEntry) {
        let item = entry.to_attribute_value_map();
        let input = PutItemInput {
            item,
            table_name: "trangar.com.requests".to_owned(),
            ..Default::default()
        };
        tokio::spawn(
            self.dynamo_db_client
                .put_item(input)
                .map(|result| println!("Result {:?}", result))
                .map_err(|e| println!("Error {:?}", e)),
        );
    }
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

impl LogEntry {
    pub fn to_attribute_value_map(&self) -> HashMap<String, AttributeValue> {
        let mut map = HashMap::new();

        map.insert(
            String::from("id"),
            AttributeValue {
                s: Some(Uuid::new_v4().to_string()),
                ..Default::default()
            },
        );
        map.insert(
            String::from("status_code"),
            AttributeValue {
                n: Some(format!("{}", self.status_code)),
                ..Default::default()
            },
        );
        map.insert(
            String::from("start"),
            AttributeValue {
                s: Some(self.start.to_rfc3339()),
                ..Default::default()
            },
        );
        map.insert(
            String::from("end"),
            AttributeValue {
                s: Some(self.end.to_rfc3339()),
                ..Default::default()
            },
        );
        if let Some(response_size) = self.response_size {
            map.insert(
                String::from("response_size"),
                AttributeValue {
                    n: Some(format!("{}", response_size)),
                    ..Default::default()
                },
            );
        }
        map
    }
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

        self.log_entry(entry);

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

/*
impl LogUploaderService {
    fn get_upload_interval() -> Duration {
        Duration::from_secs(60)
    }

    fn upload(&mut self, _: &mut Context<Self>) {
        let mut logs = self.logger.logs.lock().unwrap();

        let mut write_requests = Vec::new();
        for log in logs.drain(..) {
            write_requests.push(WriteRequest {
                delete_request: None,
                put_request: Some(PutRequest {
                    item: log.to_attribute_value_map(),
                }),
            });
        }

        let input = BatchWriteItemInput {
            request_items: {
                let mut map = HashMap::new();
                map.insert(String::from("trangar.com.requests"), write_requests);
                map
            },
            ..Default::default()
        };
        self.dynamo_db_client
            .batch_write_item(input)
            .expect("Could not write logs");
        // TODO: Upload
        logs.drain(..);
    }
}

impl ArbiterService for LogUploaderService {
    fn service_started(&mut self, ctx: &mut Self::Context) {
        ctx.run_interval(
            LogUploaderService::get_upload_interval(),
            LogUploaderService::upload,
        );
    }
}

impl Supervised for LogUploaderService {}

impl Actor for LogUploaderService {
    type Context = Context<Self>;
}
*/
