-- This file should undo anything in `up.sql`
ALTER TABLE nationalities DROP COLUMN continent;

DROP TABLE subdivisions;
DROP TYPE continent;

ALTER TABLE players DROP COLUMN subdivision;

-- Undo audit log related changes

CREATE OR REPLACE FUNCTION audit_player_modification() RETURNS trigger as $player_modification_trigger$
DECLARE
    name_change CITEXT;
    banned_change BOOLEAN;
BEGIN
    IF (OLD.name <> NEW.name) THEN
        name_change = OLD.name;
    END IF;

    IF (OLD.banned <> NEW.banned) THEN
        banned_change = OLD.banned;
    END IF;

    INSERT INTO player_modifications (userid, id, name, banned)
        (SELECT id, NEW.id, name_change, banned_change FROM active_user LIMIT 1);

    RETURN NEW;
END;
$player_modification_trigger$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION audit_player_deletion() RETURNS trigger AS $player_deletion_trigger$
BEGIN
    INSERT INTO player_modifications (userid, id, name, banned)
        (SELECT id, OLD.id, OLD.name, OLD.banned
         FROM active_user LIMIT 1);

    INSERT INTO player_deletions (userid, id)
        (SELECT id, OLD.id FROM active_user LIMIT 1);

    RETURN NULL;
END;
$player_deletion_trigger$ LANGUAGE plpgsql;

ALTER TABLE player_modifications DROP COLUMN nationality, DROP COLUMN subdivision;