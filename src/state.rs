use failure::Error;
use handlebars::{Handlebars, Helper, HelperResult, RenderContext, Renderable};
use trangarcom::DbConnection;

pub struct AppState {
    pub db: DbConnection,
    pub hbs: Handlebars,
}

pub struct StateProvider {
    db: DbConnection,
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
            "/static/",
            stringify!($file),
            ".hbs"
        )
    };
}

impl StateProvider {
    pub fn new() -> Result<StateProvider, Error> {
        let db = ::trangarcom::establish_connection()?;
        Ok(StateProvider { db })
    }

    pub fn create_state(&self) -> AppState {
        let mut hbs = Handlebars::new();
        hbs.set_strict_mode(true);
        hbs.register_helper("equals", Box::new(handlebars_equals));
        hbs.register_helper("markdown", Box::new(handlebars_markdown));

        load!(hbs, template index);
        load!(hbs, template blog);
        load!(hbs, template blog_detail);
        load!(hbs, partial layout);
        AppState {
            db: self.db.clone(),
            hbs,
        }
    }
}

fn handlebars_equals(h: &Helper, hbs: &Handlebars, rc: &mut RenderContext) -> HelperResult {
    let first = h.param(0).unwrap();
    let second = h.param(1).unwrap();

    if first.value() == second.value() {
        h.template().unwrap().render(hbs, rc)
    } else {
        Ok(())
    }
}

fn handlebars_markdown(h: &Helper, _: &Handlebars, rc: &mut RenderContext) -> HelperResult {
    use pulldown_cmark::{html, Parser};

    let value = h.param(0).unwrap().value().as_str().unwrap();
    let value = value.replace(">", "&gt;").replace("<", "&lt;");
    let parser = Parser::new(&value);

    let mut html_buf = String::new();
    html::push_html(&mut html_buf, parser);
    rc.writer.write(html_buf.as_bytes()).unwrap();
    Ok(())
}
