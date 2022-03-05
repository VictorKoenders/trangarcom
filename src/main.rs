use clap::Parser;
use std::{net::SocketAddr, sync::Arc};

mod data;
mod middleware;
mod routes;

#[async_std::main]
async fn main() -> Result<(), std::io::Error> {
    let args = Args::parse();
    let context = Context::default();
    let registry = Arc::new(context.prometheus.clone());

    let mut server = tide::with_state(context);

    server.with(middleware::Prometheus::new(&registry));

    routes::configure(&mut server);
    server.listen(args.addr).await
}

#[derive(Default, Clone)]
pub struct Context {
    pub data: data::DbContext,
    pub prometheus: prometheus::Registry,
}

#[derive(Parser)]
pub struct Args {
    /// Which address to listen on
    #[clap(short, long, default_value = "127.0.0.1:8000")]
    pub addr: SocketAddr,
}
