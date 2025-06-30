-- Add down migration script here

CREATE DOMAIN EMAIL AS CITEXT
    CHECK ( value ~ '^[a-zA-Z0-9.!#$%&''*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$' );

-- Set it to something that is most definitely not a valid bcrypt hash. The account still has the google link,
-- so password login is impossible anyway (and if it was somehow possible, this would then just result in an internal
-- server error being returned).
UPDATE members SET password_hash = 'WTF' WHERE password_hash IS NULL;

ALTER TABLE members ALTER COLUMN password_hash SET NOT NULL;
ALTER TABLE members ADD COLUMN email_address EMAIL;