use parking_lot::Mutex;
use prometheus::{
    Encoder, Histogram, HistogramOpts, HistogramTimer, IntCounterVec, Opts, Registry, TextEncoder,
};
use rocket::fairing::{Fairing, Info, Kind};
use rocket::{Data, Request, Response, State};

pub struct Prometheus {
    pub request_timer: Histogram,
    pub response: IntCounterVec,
    pub registry: Registry,
}

impl Default for Prometheus {
    fn default() -> Prometheus {
        let request_timer_opts = HistogramOpts::new("requests_timer", "Request duration");
        let request_timer = Histogram::with_opts(request_timer_opts).unwrap();

        let response_opts = Opts::new("response", "Responses");
        let response = IntCounterVec::new(response_opts, &["code"]).unwrap();

        let response_size_opts = HistogramOpts::new("response_size", "Respones size (bytes)");
        let response_size = Histogram::with_opts(response_size_opts).unwrap();

        let registry = Registry::new();
        registry.register(Box::new(request_timer.clone())).unwrap();
        registry.register(Box::new(response.clone())).unwrap();
        registry.register(Box::new(response_size.clone())).unwrap();

        Prometheus {
            request_timer,
            response,
            registry,
        }
    }
}

impl Prometheus {
    pub fn get_endpoint_contents(&self) -> Result<String, failure::Error> {
        let mut buffer = vec![];
        let encoder = TextEncoder::new();
        let metric_familys = self.registry.gather();
        encoder.encode(&metric_familys, &mut buffer).unwrap();
        Ok(String::from_utf8(buffer)?)
    }
}

pub struct PrometheusFairing;

impl Fairing for PrometheusFairing {
    fn info(&self) -> Info {
        Info {
            name: "Prometheus stats",
            kind: Kind::Request | Kind::Response,
        }
    }

    fn on_request(&self, request: &mut Request, _: &Data) {
        let state = request.guard::<State<Prometheus>>().unwrap();
        let timer = state.request_timer.start_timer();
        request.local_cache(|| PrometheusState {
            timer: Mutex::new(Some(timer)),
        });
    }

    fn on_response(&self, request: &Request, response: &mut Response) {
        let state = request.guard::<State<Prometheus>>().unwrap();
        let request_state: &PrometheusState = request.local_cache(|| unreachable!());
        state.response.with_label_values(&["all"]).inc();
        state.response
            .with_label_values(&[&response.status().code.to_string()])
            .inc();
        if let Some(timer) = request_state.timer.lock().take() {
            timer.observe_duration();
        }
    }
}

struct PrometheusState {
    timer: Mutex<Option<HistogramTimer>>,
}

