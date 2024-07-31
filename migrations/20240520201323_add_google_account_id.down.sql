-- Add down migration script here

ALTER TABLE members
    DROP COLUMN google_account_id;

ALTER TABLE members
    ALTER COLUMN password_hash SET NOT NULL;
