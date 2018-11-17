use chrono::{DateTime, Utc};
use prometheus::HistogramTimer;

pub struct LogData {
    pub start: DateTime<Utc>,
    pub timer: HistogramTimer,
}
