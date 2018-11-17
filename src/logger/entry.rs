use chrono::{DateTime, Utc};
use rusoto_dynamodb::AttributeValue;
use std::collections::HashMap;
use uuid::Uuid;

pub struct LogEntry {
    pub status_code: u16,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub response_size: Option<usize>,
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
