use rocket::http::uri::Origin;
use rocket_contrib::databases::diesel;

#[database("DATABASE")]
pub struct Database(diesel::PgConnection);

pub struct Header<'a> {
    origin: &'a Origin<'a>,
}

impl<'a> Header<'a> {
    pub fn new(origin: &'a Origin<'a>) -> Self {
        Self { origin }
    }

    pub fn active(&self, url: &str) -> &'static str {
        let is_active = (url == "/" && self.origin.path() == "/")
            || (url != "/" && self.origin.path().starts_with(url));

        if is_active {
            "active"
        } else {
            ""
        }
    }
}

pub struct Headers(pub String);

impl<'a, 'b> rocket::request::FromRequest<'a, 'b> for Headers {
    type Error = !;

    fn from_request(
        req: &'a rocket::Request<'b>,
    ) -> rocket::Outcome<Headers, (rocket::http::Status, !), ()> {
        use std::fmt::Write;

        let mut str = "{ ".to_owned();
        for (index, header) in req.headers().iter().enumerate() {
            if index != 0 {
                str += ", ";
            }
            write!(&mut str, "{:?}: {:?}", header.name(), header.value)
                .expect("Could not write to header string");
        }
        str += " }";
        rocket::Outcome::Success(Headers(str))
    }
}
