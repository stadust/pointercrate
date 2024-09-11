-- This file should undo anything in `up.sql`

ALTER TABLE players DROP COLUMN nationality;
ALTER TABLE members DROP COLUMN nationality;

DROP TABLE nationalities;
