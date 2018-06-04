-- This file should undo anything in `up.sql`
ALTER TABLE request DROP COLUMN status_code;
ALTER TABLE request ADD COLUMN remote_ip TEXT;
UPDATE request SET remote_ip = '127.0.0.1';
ALTER TABLE request ALTER COLUMN remote_ip SET NOT NULL;
