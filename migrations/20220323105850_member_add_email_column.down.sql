-- This file should undo anything in `up.sql`

ALTER TABLE members DROP COLUMN email_address;
DROP DOMAIN EMAIL;