use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use failure::Error;
use schema::portfoliopost;

use DbConnection;

#[derive(Queryable, Clone, Serialize)]
pub struct PortfolioSummary {
    pub title: String,
    pub seo_name: String,
    pub summary: String,
    pub summary_image: String,
}

impl PortfolioSummary {
    pub fn load_latest(db: &DbConnection) -> Result<Vec<PortfolioSummary>, Error> {
        let conn = db.conn.get()?;
        portfoliopost::table
            .filter(portfoliopost::dsl::published.eq(true))
            .limit(5)
            .select((
                portfoliopost::dsl::title,
                portfoliopost::dsl::seo_name,
                portfoliopost::dsl::summary,
                portfoliopost::dsl::summary_image,
            ))
            .get_results(&conn)
            .map_err(Into::into)
    }
}
