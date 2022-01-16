mod portfolio;

type Request = tide::Request<crate::Context>;

pub fn configure(app: &mut tide::Server<crate::Context>) {
    app.at("/").get(index);
    app.at("/resume").get(resume);
    app.at("/robots.txt").get(robots_txt);
    app.at("/prometheus").get(prometheus);
    app.at("/portfolio").get(portfolio::list);
    app.at("/static")
        .serve_dir("static/")
        .expect("Could not serve ./static/");
    app.at("/")
        .serve_dir("static/favicon")
        .expect("Could not serve ./static/favicon/");
}

fn respond_html_template<T: askama::Template>(t: T) -> tide::Result {
    let body = tide::Body::from_string(t.render().unwrap_or_else(|e| {
        format!(
            "<html><body><h1>Internal server error</h1><pre>{:?}</pre></body></html>",
            e
        )
    }));
    Ok(tide::Response::builder(200)
        .content_type(tide::http::mime::HTML)
        .body(body)
        .build())
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

async fn index(_: Request) -> tide::Result {
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

async fn resume(_: Request) -> tide::Result {
    respond_html_template(Resume {})
}

#[derive(askama::Template)]
#[template(path = "resume.html")]
struct Resume {}

async fn robots_txt(_: Request) -> tide::Result {
    Ok(r#"
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
    .into())
}

async fn prometheus(req: Request) -> tide::Result {
    use ::prometheus::{Encoder, TextEncoder};

    let mut buffer = Vec::new();
    let metric_families = req.state().prometheus.gather();
    TextEncoder::new()
        .encode(&metric_families, &mut buffer)
        .unwrap();
    Ok(String::from_utf8(buffer).unwrap().into())
}
