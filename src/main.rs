use actix_web::{App, HttpServer};
use std::sync::Arc;

mod data;
mod middleware;
mod routes;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let _ = dotenv::dotenv(); // this may fail, we don't always have a .env file

    let local = tokio::task::LocalSet::new();
    let sys = actix_rt::System::run_in_tokio("server", &local);

    let context = data::DbContext::default();
    let registry = Arc::new(prometheus::Registry::new());
    let prometheus = middleware::Prometheus::new(&registry);

    let addr = "0.0.0.0:8000";
    HttpServer::new(move || {
        let registry = Arc::clone(&registry);
        let prometheus = prometheus.clone();
        App::new()
            .data(context.clone())
            .app_data(registry)
            .wrap(actix_web::middleware::Compress::default())
            .wrap(prometheus)
            .configure(routes::configure)
    })
    .bind(addr)?
    .run()
    .await?;

    sys.await?;

    Ok(())
}
