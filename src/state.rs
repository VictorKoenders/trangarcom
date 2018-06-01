use trangarcom::DbConnection;
use failure::Error;
use handlebars::Handlebars;

pub struct AppState {
    pub db: DbConnection,
    pub hbs: Handlebars,
}

pub struct StateProvider {
    db: DbConnection,
}

macro_rules! load {
    ($hbs:expr, template $file:tt) => {
        $hbs.register_template_string(
            stringify!($file),
            include_str!(
                load!(url $file)
            )
        ).expect(concat!(
            "Could not load template ",
            load!(url $file)
        ));
    };
    ($hbs:expr, partial $file:tt) => {
        $hbs.register_partial(
            stringify!($file),
            include_str!(
                load!(url $file)
            )
        ).expect(concat!(
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
        Ok(StateProvider {
            db,
        })
    }

    pub fn create_state(&self) -> AppState {
        let mut hbs = Handlebars::new();
        hbs.set_strict_mode(true);

        load!(hbs, template index);
        load!(hbs, partial layout);
        AppState {
            db: self.db.clone(),
            hbs,
        }
    }
}