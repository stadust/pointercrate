-- This file should undo anything in `up.sql`


CREATE OR REPLACE FUNCTION audit_demon_modification() RETURNS trigger AS $demon_modification_trigger$
    DECLARE
        name_change CITEXT;
        position_change SMALLINT;
        requirement_change SMALLINT;
        video_change VARCHAR(200);
        verifier_change SMALLINT;
        publisher_change SMALLINT;
    BEGIN
        IF (OLD.name <> NEW.name) THEN
            name_change = OLD.name;
        END IF;

        IF (OLD.position <> NEW.position) THEN
            position_change = OLD.position;
        END IF;

        IF (OLD.requirement <> NEW.requirement) THEN
            requirement_change = OLD.requirement;
        END IF;

        IF (OLD.video <> NEW.video) THEN
            video_change = OLD.video;
        END IF;

        IF (OLD.verifier <> NEW.verifier) THEN
            verifier_change = OLD.verifier;
        END IF;

        IF (OLD.publisher <> NEW.publisher) THEN
            publisher_change = OLD.publisher;
        END IF;

        INSERT INTO demon_modifications (userid, name, position, requirement, video, verifier, publisher, id)
            (SELECT id, name_change, position_change, requirement_change, video_change, verifier_change, publisher_change, NEW.id
            FROM active_user LIMIT 1);

        RETURN NEW;
    END;
$demon_modification_trigger$ LANGUAGE plpgsql;