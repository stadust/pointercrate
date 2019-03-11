-- This file should undo anything in `up.sql`

ALTER TABLE members ALTER COLUMN password_hash TYPE BYTEA;