use prometheus::*;
use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::Status;
use rocket::request::FromRequest;
use rocket::{Data, Outcome, Request, Response};
use parking_lot::Mutex;

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
        let response = IntCounterVec::new(response_opts, &["all"]).unwrap();

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

impl Fairing for Prometheus {
    fn info(&self) -> Info {
        Info {
            name: "Prometheus stats",
            kind: Kind::Request | Kind::Response,
        }
    }

    fn on_request(&self, request: &mut Request, _: &Data) {
        let timer = self.request_timer.start_timer();
        request.local_cache(|| PrometheusState {
            registry: self.registry.clone(),
            timer: Mutex::new(Some(timer)),
        });
    }

    fn on_response(&self, request: &Request, response: &mut Response) {
        let state: &PrometheusState = request.local_cache(|| unreachable!());
        self.response.with_label_values(&["all"]).inc();
        self.response.with_label_values(&[&response.status().code.to_string()]).inc();
        if let Some(timer) = state.timer.lock().take() {
            timer.observe_duration();
        }
    }
}

struct PrometheusState {
    registry: Registry,
    timer: Mutex<Option<HistogramTimer>>,
}

pub struct PrometheusStats {
    pub registry: Registry,
}

impl<'a, 'r> FromRequest<'a, 'r> for PrometheusStats {
    type Error = !;

    fn from_request(request: &'a Request<'r>) -> Outcome<Self, (Status, Self::Error), ()> {
        let state: &PrometheusState = request
            .local_cache(|| panic!("We don't have a prometheus state, is the fairing configured?"));

        Outcome::Success(PrometheusStats {
            registry: state.registry.clone(),
        })
    }
}

