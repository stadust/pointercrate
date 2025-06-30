-- Add up migration script here

ALTER TABLE members ALTER COLUMN password_hash DROP NOT NULL;
ALTER TABLE members DROP COLUMN email_address;

DROP DOMAIN email;