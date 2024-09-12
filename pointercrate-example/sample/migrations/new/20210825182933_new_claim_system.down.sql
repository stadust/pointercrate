-- This file should undo anything in `up.sql`

ALTER TABLE members ADD COLUMN claimed_player INTEGER REFERENCES players(id) ON DELETE SET NULL;
ALTER TABLE players ADD COLUMN claimed_by INTEGER REFERENCES members(member_id) ON DELETE SET NULL;

UPDATE members SET claimed_player = (select player_id from player_claims where player_claims.member_id = members.member_id limit 1);

DROP TABLE player_claims;