-- This file should undo anything in `up.sql`

alter table members alter column password_hash type bytea using decode(password_hash, 'escape');
