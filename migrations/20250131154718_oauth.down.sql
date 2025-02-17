-- Add down migration script here
ALTER TABLE members DROP COLUMN discord_account_id;
ALTER TABLE members DROP COLUMN google_account_id;
