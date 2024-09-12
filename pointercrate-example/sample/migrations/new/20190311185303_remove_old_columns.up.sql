-- Your SQL goes here
ALTER TABLE demons DROP COLUMN notes;
ALTER TABLE demons DROP COLUMN description;

ALTER TABLE members DROP COLUMN password_salt;

ALTER TABLE members ALTER COLUMN display_name TYPE TEXT;
ALTER TABLE members ALTER COLUMN name TYPE TEXT;