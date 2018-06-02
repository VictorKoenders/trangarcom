use chrono::NaiveDate;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use diesel::sql_types::{Text, Nullable};
use failure::Error;
use schema::blogpost;

use DbConnection;

#[derive(Queryable, Serialize)]
pub struct BlogListItem {
    pub seo_name: String,
    pub title: String,
    pub date: NaiveDate,
    pub summary: String,
}

impl BlogListItem {
    pub fn load_blog_posts(db: &DbConnection, limit: i64, offset: i64) -> Result<Vec<BlogListItem>, Error> {
        let conn = db.conn.get()?;
        blogpost::table
            .filter(blogpost::dsl::published.eq(true))
            .limit(limit)
            .offset(offset)
            .select((blogpost::dsl::seo_name, blogpost::dsl::title, blogpost::dsl::date, blogpost::dsl::summary))
            .get_results(&conn)
            .map_err(Into::into)
    }
}

#[derive(QueryableByName, Serialize)]
pub struct BlogItem {
    #[sql_type = "Nullable<Text>"]
    pub previous_post_seo_name: Option<String>,
    #[sql_type = "Nullable<Text>"]
    pub previous_post_title: Option<String>,
    #[sql_type = "Nullable<Text>"]
    pub next_post_seo_name: Option<String>,
    #[sql_type = "Nullable<Text>"]
    pub next_post_title: Option<String>,

    #[sql_type = "Text"]
    pub title: String,
    #[sql_type = "Text"]
    pub content: String,
}

impl BlogItem {
    pub fn load(db: &DbConnection, name: &str) -> Result<Option<BlogItem>, Error> {
        let conn = db.conn.get()?;
        let result = ::diesel::sql_query(r#"
SELECT
	previous.seo_name AS previous_post_seo_name,
	previous.title AS previous_post_title,
	next.seo_name AS next_post_seo_name,
	next.title AS next_post_title,
	blogpost.title,
	blogpost.content
FROM blogpost
LEFT JOIN blogpost AS previous ON previous.ID = (
	SELECT ID FROM blogpost AS previous WHERE previous.published = true AND previous.date < blogpost.date ORDER BY previous.date DESC LIMIT 1
)
LEFT JOIN blogpost AS next ON next.ID = (
	SELECT ID FROM blogpost AS next WHERE next.published = true AND next.date > blogpost.date ORDER BY next.date ASC LIMIT 1
)
WHERE blogpost.seo_name = $1
"#)
        .bind::<Text, _>(name)
        .get_result(&conn);

        match result {
            Ok(v) => Ok(Some(v)),
            Err(::diesel::result::Error::NotFound) => Ok(None),
            Err(e) => Err(e.into())
        }
    }
}