-- Add down migration script here
-- Add down migration script here

ALTER TABLE members
    DROP COLUMN google_account_id,
     ADD COLUMN email_address EMAIL;
