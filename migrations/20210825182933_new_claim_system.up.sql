-- Your SQL goes here

CREATE TABLE player_claims(
    id SERIAL PRIMARY KEY, -- only used for pagination
    member_id INTEGER NOT NULL REFERENCES members(member_id) ON DELETE CASCADE,
    player_id INTEGER NOT NULL REFERENCES players(id) ON DELETE RESTRICT,
    verified BOOLEAN NOT NULL DEFAULT FALSE
);

INSERT INTO player_claims (member_id, player_id)
    SELECT member_id, claimed_player FROM members WHERE claimed_player IS NOT NULL;

ALTER TABLE members DROP COLUMN claimed_player;
ALTER TABLE players DROP COLUMN claimed_by;