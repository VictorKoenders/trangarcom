use crate::Context;
use prometheus::{Histogram, HistogramOpts, HistogramTimer, IntCounterVec, Opts, Registry};
use tide::{
    http::{Method, Url},
    Middleware, Next, Request, StatusCode,
};

pub struct Prometheus {
    metrics: Metrics,
}

impl Prometheus {
    pub fn new(registry: &Registry) -> Self {
        Self {
            metrics: Metrics::new(registry),
        }
    }
}

#[tide::utils::async_trait]
impl Middleware<Context> for Prometheus {
    async fn handle(&self, req: Request<Context>, next: Next<'_, Context>) -> tide::Result {
        self.metrics.record_method(req.method());
        let url = req.url().clone();

        let timer = self.metrics.start_response_timer();

        let res = next.run(req).await;

        self.metrics.record_response(url, res.status());
        timer.observe_duration();

        Ok(res)
    }
}

struct Metrics {
    methods: IntCounterVec,
    response_code: IntCounterVec,
    urls_with_status: IntCounterVec,
    urls: IntCounterVec,
    response_times: Histogram,
}

impl Metrics {
    pub fn new(registry: &Registry) -> Self {
        let result = Self {
            methods: IntCounterVec::new(Opts::new("methods", "HTTP methods"), &["method"]).unwrap(),
            urls: IntCounterVec::new(Opts::new("urls", "URLs requested"), &["url"]).unwrap(),
            urls_with_status: IntCounterVec::new(
                Opts::new("urls_with_status", "URLs with status"),
                &["url", "status"],
            )
            .unwrap(),
            response_code: IntCounterVec::new(
                Opts::new("response_code", "HTTP response code"),
                &["code"],
            )
            .unwrap(),
            response_times: Histogram::with_opts(HistogramOpts::new(
                "response_time",
                "Response times in ms",
            ))
            .unwrap(),
        };

        registry.register(Box::new(result.methods.clone())).unwrap();
        registry.register(Box::new(result.urls.clone())).unwrap();
        registry
            .register(Box::new(result.response_code.clone()))
            .unwrap();
        registry
            .register(Box::new(result.response_times.clone()))
            .unwrap();
        registry
            .register(Box::new(result.urls_with_status.clone()))
            .unwrap();

        result
    }

    pub fn record_method(&self, method: Method) {
        self.methods.with_label_values(&["all"]).inc();
        self.methods.with_label_values(&[&method.to_string()]).inc();
    }

    pub fn start_response_timer(&self) -> HistogramTimer {
        self.response_times.start_timer()
    }

    pub fn record_response(&self, url: Url, code: StatusCode) {
        let uri = url.to_string();
        let code = code.to_string();

        self.urls.with_label_values(&[&uri]).inc();
        self.response_code.with_label_values(&[&code]).inc();

        self.urls_with_status
            .with_label_values(&[&uri, &code])
            .inc();
    }
}
