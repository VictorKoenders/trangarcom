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
use rocket::{http::uri::Origin, request::State, response::content::Html};
use rocket_utils::{Database, Header};

pub const COOKIE_TWITTER_VISIBLE: &str = "twitter_visible";
pub const COOKIE_ANONYMIZE_LOGGING: &str = "anonymize_logging";

#[derive(Template)]
#[template(path = "index.html")]
pub struct Index<'a> {
    pub header: Header<'a>,
}

#[get("/")]
fn index(origin: &Origin, _db: Database) -> Result<Html<String>, Error> {
    Ok(Html(
        Index {
            header: Header::new(origin),
        }
        .render()?,
    ))
}

#[derive(Template)]
#[template(path = "portfolio.html")]
pub struct Portfolio<'a> {
    pub header: Header<'a>,
}

#[get("/portfolio")]
fn portfolio(origin: &Origin, _db: Database) -> Result<Html<String>, Error> {
    Ok(Html(
        Portfolio {
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
        .mount("/", routes![index, portfolio, get_prometheus])
        .mount(
            "/static",
            rocket_contrib::serve::StaticFiles::from("static"),
        )
        .launch();
    eprintln!("Rocket ended: {:?}", err);
}
