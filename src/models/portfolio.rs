use crate::schema::portfoliopost;
use diesel::{ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl};
use failure::Error;

#[derive(Queryable, Clone, Serialize)]
pub struct PortfolioSummary {
    pub title: String,
    pub seo_name: String,
    pub summary: String,
    pub summary_image: String,
}

impl PortfolioSummary {
    pub fn load_latest(conn: &PgConnection) -> Result<Vec<PortfolioSummary>, Error> {
        portfoliopost::table
            .filter(portfoliopost::dsl::published.eq(true))
            .limit(5)
            .select((
                portfoliopost::dsl::title,
                portfoliopost::dsl::seo_name,
                portfoliopost::dsl::summary,
                portfoliopost::dsl::summary_image,
            ))
            .get_results(conn)
            .map_err(Into::into)
    }
}
