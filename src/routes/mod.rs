use actix_web::{get, http::header::ContentType, web, HttpRequest, Responder};
use std::sync::Arc;

pub fn configure(cfg: &mut web::ServiceConfig) {
    // static files
    cfg.service(actix_files::Files::new("/static", "./static"))
        // routes
        .service(index)
        .service(robots_txt)
        // .service(portfolio)
        .service(prometheus);
}

fn respond_html_template<T: askama::Template>(t: T) -> impl Responder {
    match t.render() {
        Ok(body) => {
            let mut builder = web::HttpResponse::Ok();
            builder.set(ContentType::html());
            builder.body(body)
        }
        Err(e) => {
            let mut builder = web::HttpResponse::InternalServerError();
            builder.set(ContentType::html());
            builder.body(format!(
                "<html><body><h1>Internal server error</h1><pre>{:?}</pre></body></html>",
                e
            ))
        }
    }
}

pub struct Header<'a> {
    title: &'a str,
    url: &'a str,
}

impl Header<'_> {
    fn active(&self, url: &str) -> &'static str {
        if self.url == url {
            "active"
        } else {
            ""
        }
    }
}

#[get("/")]
async fn index() -> impl Responder {
    respond_html_template(Index {
        header: Header {
            title: "Trangar.com",
            url: "/",
        },
    })
}

#[derive(askama::Template)]
#[template(path = "index.html")]
struct Index<'a> {
    pub header: Header<'a>,
}

#[get("/robots.txt")]
async fn robots_txt() -> impl Responder {
    r#"
User-agent: *
Allow: *
Disallow: /admin/
Disallow: /administrator/
Disallow: /bb-admin/
Disallow: /login/
Disallow: /phpMyAdmin/
Disallow: /wp-login.php
Disallow: /wp-content/
Disallow: /wp-admin/
"#
}

/*#[get("/portfolio")]
async fn portfolio() -> impl Responder {
    respond_html_template(Portfolio {
        header: Header {
            title: "Trangar.com",
            url: "/portfolio",
        },
    })
}

#[derive(askama::Template)]
#[template(path = "portfolio.html")]
struct Portfolio<'a> {
    pub header: Header<'a>,
}*/

#[get("/prometheus")]
async fn prometheus(request: HttpRequest) -> impl Responder {
    use ::prometheus::{Encoder, Registry, TextEncoder};

    let mut buffer = Vec::new();
    let metric_families = request.app_data::<Arc<Registry>>().unwrap().gather();
    TextEncoder::new()
        .encode(&metric_families, &mut buffer)
        .unwrap();
    String::from_utf8(buffer).unwrap()
}
