-- This file should undo anything in `up.sql`
ALTER TABLE demons ADD COLUMN notes TEXT NULL;
ALTER TABLE demons ADD COLUMN description TEXT NULL;

ALTER TABLE members ADD COLUMN password_salt BYTEA NOT NULL DEFAULT E'\\000';

ALTER TABLE members ALTER COLUMN display_name TYPE CITEXT;
ALTER TABLE members ALTER COLUMN name TYPE CITEXT;