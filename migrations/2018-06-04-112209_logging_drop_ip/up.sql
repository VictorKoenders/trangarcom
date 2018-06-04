-- Your SQL goes here
ALTER TABLE request DROP COLUMN remote_ip;
ALTER TABLE request ADD COLUMN status_code SMALLINT;
