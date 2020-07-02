use actix_web::dev::*;
use actix_web::Error;
use appinsights::{
    telemetry::RequestTelemetry, telemetry::Telemetry, InMemoryChannel, TelemetryClient,
};
use futures::{
    future::{ok, Ready},
    Future,
};
use std::{pin::Pin, sync::Arc, time::Instant};

pub struct AppInsights {
    client: Arc<TelemetryClient<InMemoryChannel>>,
}

impl AppInsights {
    pub fn new(client: Arc<TelemetryClient<InMemoryChannel>>) -> Self {
        Self { client }
    }
}

impl<S, B> Transform<S> for AppInsights
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = AppInsightsMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(AppInsightsMiddleware {
            service,
            client: Arc::clone(&self.client),
        })
    }
}

pub struct AppInsightsMiddleware<S> {
    service: S,
    client: Arc<TelemetryClient<InMemoryChannel>>,
}

impl<S, B> Service for AppInsightsMiddleware<S>
where
    S::Future: 'static,
    B: MessageBody + 'static,
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(
        &mut self,
        ctx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }
    fn call(&mut self, req: Self::Request) -> Self::Future {
        let method = req.method().clone();
        let ip = req
            .connection_info()
            .remote()
            .unwrap_or("unknown")
            .to_string();
        let uri = req.uri().clone();
        let start = Instant::now();
        let client = Arc::clone(&self.client);

        let fut = self.service.call(req);
        Box::pin(async move {
            let res = fut.await;

            let response_code = match &res {
                Ok(response) => response.status(),
                Err(e) => e.as_response_error().status_code(),
            }
            .as_u16();

            let mut request = RequestTelemetry::new(
                method,
                uri.clone(),
                start.elapsed(),
                response_code.to_string(),
            );
            {
                let tags = request.tags_mut();
                tags.location_mut().set_ip(ip);
                tags.operation_mut().set_name(uri.to_string());
            }

            client.track(request);

            res
        })
    }
}
