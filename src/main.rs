use std::sync::Arc;

mod data;
mod middleware;
mod routes;

#[async_std::main]
async fn main() -> Result<(), std::io::Error> {
    let context = Context::default();
    let registry = Arc::new(context.prometheus.clone());

    let addr = "0.0.0.0:8000";

    let mut server = tide::with_state(context);

    server.with(middleware::Prometheus::new(&registry));

    routes::configure(&mut server);
    server.listen(addr).await
}

#[derive(Default, Clone)]
pub struct Context {
    pub data: data::DbContext,
    pub prometheus: prometheus::Registry,
}
