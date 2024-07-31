-- Add up migration script here

ALTER TABLE members
    ADD COLUMN google_account_id VARCHAR(255) NULL;

ALTER TABLE members
    ALTER COLUMN password_hash DROP NOT NULL;
