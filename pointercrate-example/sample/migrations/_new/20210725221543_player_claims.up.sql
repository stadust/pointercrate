-- Your SQL goes here

ALTER TABLE members ADD COLUMN claimed_player INTEGER REFERENCES players(id) ON DELETE SET NULL;
ALTER TABLE players ADD COLUMN claimed_by INTEGER REFERENCES members(member_id) ON DELETE SET NULL;
