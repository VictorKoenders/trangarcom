-- Your SQL goes here
CREATE TABLE PortfolioPost (
    id UUID PRIMARY KEY DEFAULT (uuid_generate_v4()),
    "date" DATE NOT NULL,
    published BOOLEAN NOT NULL,
    seo_name TEXT UNIQUE NOT NULL,
    title TEXT NOT NULL,
    summary TEXT NOT NULL,
    summary_image TEXT NOT NULL,
    content TEXT NOT NULL
);

