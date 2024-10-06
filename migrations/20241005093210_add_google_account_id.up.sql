-- Add up migration script here
-- Add up migration script here

ALTER TABLE members
    ADD COLUMN google_account_id VARCHAR(255) UNIQUE NULL,
    DROP COLUMN email_address;
