extern crate actix_web;
extern crate actix;
extern crate chrono;
extern crate time;
extern crate trangarcom;
extern crate failure;
extern crate uuid;
extern crate futures;
extern crate handlebars;
#[macro_use]
extern crate serde_json;

mod logger;
mod state;

use actix_web::{server, App, HttpRequest, HttpResponse};
use state::{AppState, StateProvider};
use logger::Logger;

fn index(req: HttpRequest<AppState>) -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html")
        .body(
            req.state().hbs.render("index", &()).expect("Could not render template \"index\"")
        )
}

fn main() -> Result<(), failure::Error> {
    let sys = actix::System::new("trangarcom");
    let state_provider = StateProvider::new()?;
    server::new(move || {
        let logger = Logger::default();
        App::with_state(state_provider.create_state())
            .middleware(logger)
            .resource("/", |r| r.f(index))
            // .resource("/", |r| r.f(greet))
            // .resource("/{name}", |r| r.f(greet))
    })
    .bind("0.0.0.0:8000")
        .expect("Can not bind to port 8000")
        .start();

    sys.run();
    Ok(())
}
