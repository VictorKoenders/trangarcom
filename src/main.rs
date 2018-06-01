extern crate actix;
extern crate actix_web;
extern crate chrono;
extern crate failure;
extern crate futures;
extern crate handlebars;
extern crate time;
extern crate trangarcom;
extern crate uuid;

mod logger;
mod state;

use actix_web::{server, App, HttpRequest, HttpResponse};
use logger::Logger;
use state::{AppState, StateProvider};
use std::collections::BTreeMap;

fn index(req: HttpRequest<AppState>) -> HttpResponse {
    HttpResponse::Ok().content_type("text/html").body(
        req.state()
            .hbs
            .render("index", &())
            .expect("Could not render template \"index\""),
    )
}

fn blog_list(req: HttpRequest<AppState>) -> HttpResponse {
    let list = vec![
        trangarcom::models::BlogListItem {
            seo_name: String::from("neural_network_on_micro_controller"),
            title: String::from("Running a neural network on a micro controller"),
            date: String::from("May 23, 2018"),
            summary: String::from("I've been playing around with the idea of running a neural network on a micro controller")
        }
    ];
    let mut data = BTreeMap::new();
    data.insert("blog_items".to_string(), list);
    HttpResponse::Ok().content_type("text/html").body(
        req.state()
            .hbs
            .render("blog", &data)
            .expect("Could not render template \"blog\""),
    )
}
fn blog_detail(req: HttpRequest<AppState>) -> HttpResponse {
    let item = trangarcom::models::BlogItem {
        title: String::from("Running a neural network on a micro controller"),
        content: String::from("Content goes here"),
        next_post_seo_name: None,
        next_post_title: None,
        previous_post_seo_name: None,
        previous_post_title: None,
    };
    HttpResponse::Ok().content_type("text/html").body(
        req.state()
            .hbs
            .render("blog_detail", &item)
            .expect("Could not render template \"blog_detail\""),
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
            .resource("/blog/{seo_name}", |r| r.f(blog_detail))
            .resource("/blog", |r| r.f(blog_list))
        // .resource("/", |r| r.f(greet))
        // .resource("/{name}", |r| r.f(greet))
    }).bind("0.0.0.0:8000")
        .expect("Can not bind to port 8000")
        .start();

    sys.run();
    Ok(())
}
