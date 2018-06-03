extern crate actix;
extern crate actix_web;
extern crate chrono;
extern crate dotenv;
extern crate failure;
extern crate futures;
extern crate handlebars;
extern crate pulldown_cmark;
extern crate time;
extern crate trangarcom;
extern crate uuid;
#[macro_use]
extern crate serde_derive;
extern crate serde;

mod logger;
mod state;

use actix_web::{http, server, App, Form, HttpRequest, HttpResponse};
use logger::Logger;
use state::{AppState, StateProvider};
use std::collections::BTreeMap;

#[derive(Deserialize, Debug)]
pub struct PrivacySettings {
    pub load_twitter: Option<u8>,
    pub anonymize_logging: Option<u8>,
}

#[derive(Serialize)]
pub struct IndexValues {
    pub load_twitter: bool,
    pub anonymize_logging: bool,
    pub remote_addr: String,
    pub headers: String,
    pub url: String,
}

fn index(mut req: HttpRequest<AppState>) -> HttpResponse {
    let values = IndexValues {
        load_twitter: req.cookie("twitter_visible").is_some(),
        anonymize_logging: req.cookie("anonymize_logging").is_some(),
        remote_addr: req.peer_addr().map(|a| a.to_string()).unwrap_or_else(||String::new()),
        headers: format!("{:?}", req.headers_mut()),
        url: req.uri().to_string(),
    };
    HttpResponse::Ok().content_type("text/html").body(
        req.state()
            .hbs
            .render("index", &values)
            .expect("Could not render template \"index\""),
    )
}

fn index_post((form, req): (Form<PrivacySettings>, HttpRequest<AppState>)) -> HttpResponse {
    let mut response = HttpResponse::Found();
    response.header("location", "/");
    let cookie = req.cookie("twitter_visible");
    match (cookie, form.load_twitter) {
        (Some(ref c), None) => {
            response.del_cookie(c);
        }
        (None, Some(n)) if n > 0 => {
            response.cookie(http::Cookie::build("twitter_visible", "1").finish());
        }
        _ => {}
    }
    let cookie = req.cookie("anonymize_logging");
    match (cookie, form.anonymize_logging) {
        (Some(ref c), None) => {
            response.del_cookie(c);
        }
        (None, Some(n)) if n > 0 => {
            response.cookie(http::Cookie::build("anonymize_logging", "1").finish());
            if let Some(addr) = req.peer_addr() {
                ::trangarcom::models::Request::remove_requests(&req.state().db, &addr.ip().to_string()).unwrap();
            }
        }
        _ => {}
    }
    response.finish()
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
        None => {
            return HttpResponse::MovedPermanently()
                .header("Location", "/blog")
                .finish()
        }
    };
    match trangarcom::models::BlogItem::load(&req.state().db, &name)
        .expect("Could not load blog item")
    {
        Some(item) => HttpResponse::Ok().content_type("text/html").body(
            req.state()
                .hbs
                .render("blog_detail", &item)
                .expect("Could not render template \"blog_detail\""),
        ),
        None => HttpResponse::MovedPermanently()
            .header("Location", "/blog")
            .finish(),
    }
}

fn resume(req: HttpRequest<AppState>) -> HttpResponse {
    HttpResponse::Ok().content_type("text/html").body(
        req.state()
            .hbs
            .render("resume", &())
            .expect("Could not render template \"resume\""),
    )
}

fn main() -> Result<(), failure::Error> {
    dotenv::dotenv()?;

    let sys = actix::System::new("trangarcom");
    let state_provider = StateProvider::new()?;
    server::new(move || {
        let logger = Logger::default();
        App::with_state(state_provider.create_state())
            .middleware(logger)
            .resource("/", |r| {
                r.get().f(index);
                r.post().with(index_post);
            })
            .resource("/blog/{seo_name}", |r| r.f(blog_detail))
            .resource("/blog", |r| r.f(blog_list))
            .resource("/resume", |r| r.f(resume))
            .handler("/images", actix_web::fs::StaticFiles::new("images"))
    }).bind("0.0.0.0:8000")
        .expect("Can not bind to port 8000")
        .start();

    sys.run();
    Ok(())
}
