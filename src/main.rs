#![feature(proc_macro_hygiene, decl_macro, never_type)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;

mod fairings;
mod rocket_utils;

use crate::fairings::Prometheus;
use askama::Template;
use failure::Error;
use rocket::http::{uri::Origin, Cookie, Cookies, RawStr};
use rocket::request::{Form, State};
use rocket::response::{content::Html, Redirect};
use rocket_utils::{Database, Header, Headers};

pub const COOKIE_TWITTER_VISIBLE: &str = "twitter_visible";
pub const COOKIE_ANONYMIZE_LOGGING: &str = "anonymize_logging";

#[derive(FromForm, Debug)]
pub struct PrivacySettings {
    pub load_twitter: bool,
    pub anonymize_logging: bool,
}

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexValues<'a> {
    pub header: Header<'a>,
    pub load_twitter: bool,
    pub anonymize_logging: bool,
    pub headers: String,
    pub url: String,
}

#[get("/")]
fn index(
    origin: &Origin,
    _db: Database,
    headers: Headers,
    cookie: Cookies,
) -> Result<Html<String>, Error> {
    let values = IndexValues {
        header: Header::new(origin),
        load_twitter: cookie.get(COOKIE_TWITTER_VISIBLE).is_some(),
        anonymize_logging: cookie.get(COOKIE_ANONYMIZE_LOGGING).is_some(),
        headers: headers.0,
        url: origin.to_string(),
    };
    Ok(Html(values.render()?))
}

#[post("/", data = "<data>")]
fn index_post(data: Form<PrivacySettings>, mut cookies: Cookies) -> Redirect {
    match (
        cookies.get(COOKIE_TWITTER_VISIBLE).is_some(),
        data.load_twitter,
    ) {
        (true, false) => {
            cookies.remove(Cookie::named(COOKIE_TWITTER_VISIBLE));
        }
        (false, true) => {
            cookies.add(Cookie::new(COOKIE_TWITTER_VISIBLE, "1"));
        }
        _ => {}
    }
    match (
        cookies.get(COOKIE_ANONYMIZE_LOGGING).is_some(),
        data.anonymize_logging,
    ) {
        (true, false) => {
            cookies.remove(Cookie::named(COOKIE_ANONYMIZE_LOGGING));
        }
        (false, true) => {
            cookies.add(Cookie::new(COOKIE_ANONYMIZE_LOGGING, "1"));
        }
        _ => {}
    }
    Redirect::to("/")
}

#[derive(Template)]
#[template(path = "blog.html")]
pub struct BlogPosts<'a> {
    pub header: Header<'a>,
    pub blog_items: Vec<BlogPost<'a>>,
}

pub struct BlogPost<'a> {
    pub title: &'a str,
    pub seo_name: &'a str,
    pub date: &'a chrono::NaiveDate,
    pub summary: &'a str,
}

impl<'a> From<&'a trangarcom::models::BlogListItem> for BlogPost<'a> {
    fn from(item: &'a trangarcom::models::BlogListItem) -> BlogPost<'a> {
        BlogPost {
            title: &item.title,
            seo_name: &item.seo_name,
            date: &item.date,
            summary: &item.summary,
        }
    }
}

#[get("/blog")]
fn blog_list(origin: &Origin, db: Database) -> Result<Html<String>, Error> {
    let items = trangarcom::models::BlogListItem::load_blog_posts(&db, 10, 0).unwrap();
    Ok(Html(
        BlogPosts {
            header: Header::new(origin),
            blog_items: items.iter().map(Into::into).collect(),
        }
        .render()?,
    ))
}

#[derive(Template)]
#[template(path = "blog_detail.html")]
pub struct BlogDetail<'a> {
    pub header: Header<'a>,
    pub blog: trangarcom::models::BlogItem,
}

impl BlogDetail<'_> {
    pub fn render_content(&self) -> String {
        let parser = pulldown_cmark::Parser::new(&self.blog.content);

        let mut html_buf = String::new();
        pulldown_cmark::html::push_html(&mut html_buf, parser);

        html_buf
    }
}

#[get("/blog/<seo_name>")]
fn blog_detail(seo_name: &RawStr, origin: &Origin, db: Database) -> Result<Html<String>, Error> {
    let blog = trangarcom::models::BlogItem::load(&db, seo_name.as_str())?
        .ok_or_else(|| failure::format_err!("Could not find blogitem"))?;
    Ok(Html(
        BlogDetail {
            header: Header::new(origin),
            blog,
        }
        .render()?,
    ))
}

#[derive(Template)]
#[template(path = "resume.html")]
struct ResumeView<'a> {
    header: Header<'a>,
}

#[get("/resume")]
fn resume(origin: &Origin) -> Result<Html<String>, Error> {
    Ok(Html(
        ResumeView {
            header: Header::new(origin),
        }
        .render()?,
    ))
}

#[get("/prometheus")]
fn get_prometheus(state: State<Prometheus>) -> Result<String, Error> {
    state.get_endpoint_contents()
}

fn main() {
    if let Err(e) = dotenv::dotenv() {
        println!("Could not load .env: {:?}", e);
    }
    use rocket::config::{Config, Environment, Value};
    use std::collections::HashMap;

    let mut database_config = HashMap::new();
    let mut databases = HashMap::new();

    // This is the same as the following TOML:
    //     // my_db = { url = "database.sqlite" }
    database_config.insert(
        "url",
        Value::from(std::env::var("DATABASE_URL").expect("DATABASE_URL not set")),
    );
    databases.insert("DATABASE", Value::from(database_config));
    let config = Config::build(Environment::Development)
        .address("0.0.0.0")
        .extra("databases", databases)
        .finalize()
        .unwrap();
    let err = rocket::custom(config)
        .attach(Database::fairing())
        .attach(fairings::Logger)
        .attach(fairings::PrometheusFairing)
        .manage(fairings::Prometheus::default())
        .mount(
            "/",
            routes![
                index,
                index_post,
                blog_list,
                blog_detail,
                resume,
                get_prometheus
            ],
        )
        .mount(
            "/static",
            rocket_contrib::serve::StaticFiles::from("static"),
        )
        .launch();
    eprintln!("Rocket ended: {:?}", err);

    /*let sys = actix::System::new("trangarcom");
    let state_provider = StateProvider::new()?;
    server::new(move || {
        let logger = Logger::default();
        App::with_state(state_provider.create_state())
            .middleware(logger)
            .resource("/", |r| {
                r.get().f(index);
                r.post().with(index_post);
            })
            .resource("/prometheus", |r| r.f(get_prometheus))
            .resource("/blog/{seo_name}", |r| r.f(blog_detail))
            .resource("/blog", |r| r.f(blog_list))
            .resource("/resume", |r| r.f(resume))
            .handler(
                "/images",
                actix_web::fs::StaticFiles::new("images").unwrap(),
            )
            .handler(
                "/static",
                actix_web::fs::StaticFiles::new("static").unwrap(),
            )
    })
    .bind("0.0.0.0:8000")
    .expect("Can not bind to port 8000")
    .start();

    sys.run();
    Ok(())*/
}
