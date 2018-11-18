use failure::Error;
use handlebars::{Context, Handlebars, Helper, HelperResult, Output, RenderContext, Renderable};
use prometheus::{Histogram, HistogramOpts, IntCounterVec, Opts, Registry};
use serde_json::Value;
use std::sync::Arc;
use trangarcom::DbConnection;

#[derive(Clone)]
pub struct Prometheus {
    pub request_timer: Histogram,
    pub response: IntCounterVec,
    pub response_size: Histogram,
    pub registry: Registry,
}

impl Default for Prometheus {
    fn default() -> Prometheus {
        let request_timer_opts = HistogramOpts::new("requests_timer", "Request duration");
        let request_timer = Histogram::with_opts(request_timer_opts).unwrap();

        let response_opts = Opts::new("response", "Responses");
        let response = IntCounterVec::new(response_opts, &["all"]).unwrap();

        let response_size_opts = HistogramOpts::new("response_size", "Respones size (bytes)");
        let response_size = Histogram::with_opts(response_size_opts).unwrap();

        let registry = Registry::new();
        registry.register(Box::new(request_timer.clone())).unwrap();
        registry.register(Box::new(response.clone())).unwrap();
        registry.register(Box::new(response_size.clone())).unwrap();

        Prometheus {
            request_timer,
            response,
            response_size,
            registry,
        }
    }
}
pub struct AppState {
    pub db: Arc<DbConnection>,
    pub hbs: Handlebars,
    pub prometheus: Arc<Prometheus>,
}

#[derive(Clone)]
pub struct StateProvider {
    db: Arc<DbConnection>,
    prometheus: Arc<Prometheus>,
}

macro_rules! load {
    ($hbs:expr,template $file:tt) => {
        $hbs.register_template_string(stringify!($file), include_str!(load!(url $file)))
            .expect(concat!("Could not load template ", load!(url $file)));
    };
    ($hbs:expr,partial $file:tt) => {
        $hbs.register_partial(stringify!($file), include_str!(load!(url $file)))
            .expect(concat!(
                "Could not load partial template ",
                load!(url $file)
            ));
    };
    (url $file:tt) => {
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/templates/",
            stringify!($file),
            ".hbs"
        )
    };
}

impl StateProvider {
    pub fn new() -> Result<StateProvider, Error> {
        let db = ::trangarcom::establish_connection()?;
        Ok(StateProvider {
            db: Arc::new(db),
            prometheus: Arc::new(Prometheus::default()),
        })
    }

    pub fn create_state(&self) -> AppState {
        let mut hbs = Handlebars::new();
        hbs.set_strict_mode(true);
        hbs.register_helper("equals", Box::new(handlebars_equals));
        hbs.register_helper("markdown", Box::new(handlebars_markdown));
        hbs.register_helper("if_is_even", Box::new(handlebars_is_even));
        hbs.register_helper("if_is_odd", Box::new(handlebars_is_odd));

        load!(hbs, template index);
        load!(hbs, template blog);
        load!(hbs, template blog_detail);
        load!(hbs, template resume);
        load!(hbs, partial layout);
        AppState {
            db: self.db.clone(),
            hbs,
            prometheus: self.prometheus.clone(),
        }
    }
}

fn handlebars_equals<'reg, 'rc>(
    h: &Helper<'reg, 'rc>,
    hbs: &'reg Handlebars,
    context: &Context,
    rc: &mut RenderContext<'reg>,
    out: &mut Output,
) -> HelperResult {
    let first = h.param(0).unwrap();
    let second = h.param(1).unwrap();

    if first.value() == second.value() {
        h.template().unwrap().render(hbs, context, rc, out)
    } else {
        Ok(())
    }
}

fn handlebars_is_even<'reg, 'rc>(
    h: &Helper<'reg, 'rc>,
    hbs: &'reg Handlebars,
    context: &Context,
    rc: &mut RenderContext<'reg>,
    out: &mut Output,
) -> HelperResult {
    let first = h.param(0).unwrap();

    if let Value::Number(n) = first.value() {
        if let Some(n) = n.as_i64() {
            if n % 2 == 0 {
                return h.template().unwrap().render(hbs, context, rc, out);
            }
        }
    }
    Ok(())
}
fn handlebars_is_odd<'reg, 'rc>(
    h: &Helper<'reg, 'rc>,
    hbs: &'reg Handlebars,
    context: &Context,
    rc: &mut RenderContext<'reg>,
    out: &mut Output,
) -> HelperResult {
    let first = h.param(0).unwrap();

    if let Value::Number(n) = first.value() {
        if let Some(n) = n.as_i64() {
            if n % 2 == 1 {
                return h.template().unwrap().render(hbs, context, rc, out);
            }
        }
    }
    Ok(())
}

fn handlebars_markdown<'reg, 'rc>(
    h: &Helper<'reg, 'rc>,
    _: &'reg Handlebars,
    _: &Context,
    _: &mut RenderContext<'reg>,
    out: &mut Output,
) -> HelperResult {
    use pulldown_cmark::{html, Parser};

    let value = h.param(0).unwrap().value().as_str().unwrap();
    let value = value.replace(">", "&gt;").replace("<", "&lt;");
    let parser = Parser::new(&value);

    let mut html_buf = String::new();
    html::push_html(&mut html_buf, parser);
    out.write(&html_buf)?;
    Ok(())
}
