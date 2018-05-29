extern crate actix_web;
extern crate actix;
extern crate chrono;
extern crate time;
extern crate trangarcom;
extern crate uuid;

mod logger;
mod state;

use actix_web::{server, App, HttpRequest, Result};
use actix_web::fs::NamedFile;
use state::{AppState, StateProvider};
use logger::Logger;

fn index(_req: HttpRequest<AppState>) -> Result<NamedFile> {
    NamedFile::open("static/index.html").map_err(Into::into)
}

fn main() {
    let sys = actix::System::new("trangarcom");
    let state_provider = StateProvider::new();
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

    let _ = sys.run();
}
