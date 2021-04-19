-- This file should undo anything in `up.sql`
ALTER TABLE nationalities DROP COLUMN continent;

DROP TABLE subdivisions;
DROP TYPE continent;

ALTER TABLE players DROP COLUMN subdivision;
