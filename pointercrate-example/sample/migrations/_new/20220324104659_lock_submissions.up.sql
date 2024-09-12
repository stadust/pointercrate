-- Your SQL goes here

ALTER TABLE player_claims ADD COLUMN lock_submissions BOOL NOT NULL DEFAULT FALSE;