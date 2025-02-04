-- Add up migration script here
ALTER TABLE members ADD COLUMN discord_account_id VARCHAR(256);
ALTER TABLE members ADD COLUMN google_account_id VARCHAR(256);
