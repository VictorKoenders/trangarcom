-- Your SQL goes here
CREATE TABLE request (
    id UUID PRIMARY KEY DEFAULT (uuid_generate_v4()),
    time TIMESTAMP NOT NULL,
    url TEXT NOT NULL,
    remote_ip TEXT NOT NULL,
    headers TEXT NOT NULL,
    response_time DOUBLE PRECISION NULL,
    finish_time DOUBLE PRECISION NULL
)

