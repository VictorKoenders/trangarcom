extern crate actix;
extern crate actix_web;
extern crate chrono;
extern crate failure;
extern crate futures;
extern crate handlebars;
extern crate time;
extern crate trangarcom;
extern crate uuid;
extern crate pulldown_cmark;

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
    let items = trangarcom::models::BlogListItem::load_blog_posts(&req.state().db, 10, 0).unwrap();

    let mut data = BTreeMap::new();
    data.insert("blog_items".to_string(), items);

    HttpResponse::Ok().content_type("text/html").body(
        req.state()
            .hbs
            .render("blog", &data)
            .expect("Could not render template \"blog\""),
    )
}
fn blog_detail(req: HttpRequest<AppState>) -> HttpResponse {
    let name = match req.match_info().get("seo_name") {
        Some(name) => name,
        None => return HttpResponse::MovedPermanently()
                        .header("Location", "/blog")
                        .finish()
    };
    match trangarcom::models::BlogItem::load(&req.state().db, &name).expect("Could not load blog item") {
        Some(item) => {
            HttpResponse::Ok().content_type("text/html").body(
                req.state()
                    .hbs
                    .render("blog_detail", &item)
                    .expect("Could not render template \"blog_detail\""),
            )
        }
        None => {
            HttpResponse::MovedPermanently()
            .header("Location", "/blog")
            .finish()
        }
    }
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
