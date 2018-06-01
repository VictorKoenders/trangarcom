#[derive(Queryable, Serialize)]
pub struct BlogListItem {
    pub seo_name: String,
    pub title: String,
    pub date: String,
    pub summary: String,
}

#[derive(Queryable, Serialize)]
pub struct BlogItem {
    pub previous_post_seo_name: Option<String>,
    pub previous_post_title: Option<String>,
    pub next_post_seo_name: Option<String>,
    pub next_post_title: Option<String>,

    pub title: String,
    pub content: String,
}
