-- This file should undo anything in `up.sql`

ALTER TABLE players DROP COLUMN nationality;
ALTER TABLE members DROP COLUMN nationality;

<<<<<<< HEAD
DROP TABLE nationalities;
=======
DROP TABLE nationality;
>>>>>>> 5b6423a... Essentially basic nationality support
