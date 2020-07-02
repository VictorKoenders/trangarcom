use actix_web::dev::*;
use actix_web::{
    http::{Method, Uri},
    Error,
};
use futures::{
    future::{ok, Ready},
    Future,
};
use prometheus::{Histogram, HistogramOpts, HistogramTimer, IntCounterVec, Opts, Registry};
use std::{pin::Pin, sync::Arc};

#[derive(Clone)]
pub struct Prometheus {
    metrics: Arc<Metrics>,
}

impl Prometheus {
    pub fn new(registry: &Registry) -> Self {
        Self {
            metrics: Metrics::new_arc(registry),
        }
    }
}

impl<S, B> Transform<S> for Prometheus
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = PrometheusMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(PrometheusMiddleware {
            service,
            metrics: Arc::clone(&self.metrics),
        })
    }
}

pub struct PrometheusMiddleware<S> {
    service: S,
    metrics: Arc<Metrics>,
}

type PinFut<Output> = Pin<Box<dyn Future<Output = Output>>>;

impl<S, B> Service for PrometheusMiddleware<S>
where
    S::Future: 'static,
    B: MessageBody + 'static,
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = PinFut<Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        ctx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }
    fn call(&mut self, req: Self::Request) -> Self::Future {
        self.metrics.record_method(req.method());
        let url = req.uri().clone();

        let timer = self.metrics.start_response_timer();
        let metrics = Arc::clone(&self.metrics);

        let fut = self.service.call(req);
        Box::pin(async move {
            let res = fut.await;

            let response_code = match &res {
                Ok(response) => response.status(),
                Err(e) => e.as_response_error().status_code(),
            }
            .as_u16();

            metrics.record_response(url, response_code);
            timer.observe_duration();

            res
        })
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
    pub fn new_arc(registry: &Registry) -> Arc<Self> {
        let result = Arc::new(Self {
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
        });

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

    pub fn record_method(&self, method: &Method) {
        self.methods.with_label_values(&["all"]).inc();
        self.methods.with_label_values(&[&method.to_string()]).inc();
    }

    pub fn start_response_timer(&self) -> HistogramTimer {
        self.response_times.start_timer()
    }

    pub fn record_response(&self, uri: Uri, code: u16) {
        let uri = uri.to_string();
        let code = code.to_string();

        self.urls.with_label_values(&[&uri]).inc();
        self.response_code.with_label_values(&[&code]).inc();

        self.urls_with_status
            .with_label_values(&[&uri, &code])
            .inc();
    }
}
