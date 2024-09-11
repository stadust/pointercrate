-- This file should undo anything in `up.sql`
CREATE OR REPLACE FUNCTION audit_user_modification() RETURNS trigger as $user_modification_trigger$
DECLARE
    display_name_change CITEXT;
    youtube_channel_change BOOLEAN;
    permissions_change BIT(16);
BEGIN
    IF (OLD.display_name <> NEW.display_name) THEN
        display_name_change = OLD.display_name;
    END IF;

    IF (OLD.youtube_channel <> NEW.youtube_channel) THEN
        youtube_channel_change = OLD.youtube_channel;
    END IF;

    IF (OLD.permissions <> NEW.permissions) THEN
        permissions_change = OLD.permissions;
    END IF;

    INSERT INTO user_modifications (userid, id, display_name, youtube_channel, permissions)
        (SELECT id, NEW.member_id, display_name_change, youtube_channel_change, permissions_change FROM active_user LIMIT 1);

    RETURN NEW;
END;
$user_modification_trigger$ LANGUAGE plpgsql;