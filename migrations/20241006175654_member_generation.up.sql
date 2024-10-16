-- Add up migration script here
ALTER TABLE members ADD COLUMN generation BIGINT NOT NULL DEFAULT 0;