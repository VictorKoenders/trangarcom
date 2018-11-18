use actix_web::http::{HeaderMap, Uri};
use chrono::{DateTime, Utc};
use rusoto_dynamodb::AttributeValue;
use std::collections::HashMap;
use uuid::Uuid;

pub struct LogEntry<'a> {
    pub status_code: u16,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub response_size: Option<usize>,
    pub uri: &'a Uri,
    pub headers: &'a HeaderMap,
}

fn attribute_string(s: String) -> AttributeValue {
    AttributeValue {
        s: Some(s),
        ..Default::default()
    }
}
fn attribute_number(n: impl std::fmt::Display) -> AttributeValue {
    AttributeValue {
        n: Some(format!("{}", n)),
        ..Default::default()
    }
}

impl<'a> LogEntry<'a> {
    pub fn to_attribute_value_map(&self) -> HashMap<String, AttributeValue> {
        let mut map = HashMap::new();

        map.insert(
            String::from("id"),
            attribute_string(Uuid::new_v4().to_string()),
        );
        map.insert(
            String::from("status_code"),
            attribute_number(self.status_code),
        );
        map.insert(
            String::from("start"),
            attribute_string(self.start.to_rfc3339()),
        );
        map.insert(String::from("end"), attribute_string(self.end.to_rfc3339()));
        map.insert(String::from("uri"), attribute_string(self.uri.to_string()));
        if let Some(response_size) = self.response_size {
            map.insert(
                String::from("response_size"),
                attribute_number(response_size),
            );
        }
        let mut header_map = HashMap::new();
        for (key, value) in self.headers {
            let value = match value.to_str() {
                Ok(s) => attribute_string(s.to_owned()),
                Err(_) => AttributeValue {
                    b: Some(value.as_bytes().to_vec()),
                    ..Default::default()
                },
            };
            header_map.insert(key.as_str().to_owned(), value);
        }
        map.insert(
            String::from("headers"),
            AttributeValue {
                m: Some(header_map),
                ..Default::default()
            },
        );
        map
    }
}
