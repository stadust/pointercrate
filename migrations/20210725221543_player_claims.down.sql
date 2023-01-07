-- This file should undo anything in `up.sql`

ALTER TABLE players DROP COLUMN claimed_by;
ALTER TABLE members DROP COLUMN claimed_player;