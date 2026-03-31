ALTER TABLE demon_modifications ADD COLUMN level_id BIGINT NULL DEFAULT NULL;

CREATE OR REPLACE FUNCTION audit_demon_modification() RETURNS trigger AS $demon_modification_trigger$
DECLARE
    name_change CITEXT;
    position_change SMALLINT;
    requirement_change SMALLINT;
    video_change VARCHAR(200);
    thumbnail_change TEXT;
    verifier_change INT;
    publisher_change INT;
    level_id_change BIGINT;
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

    IF (OLD.thumbnail <> NEW.thumbnail) THEN
        thumbnail_change = OLD.thumbnail;
    END IF;

    IF (OLD.verifier <> NEW.verifier) THEN
        verifier_change = OLD.verifier;
    END IF;

    IF (OLD.publisher <> NEW.publisher) THEN
        publisher_change = OLD.publisher;
    END IF;

    IF (OLD.level_id <> NEW.level_id) THEN
        level_id_change = OLD.level_id;
    END IF;

    INSERT INTO demon_modifications (userid, name, position, requirement, video, verifier, publisher, thumbnail, level_id, id)
        (SELECT id, name_change, position_change, requirement_change, video_change, verifier_change, publisher_change, thumbnail_change, level_id_change, NEW.id
         FROM active_user LIMIT 1);

    RETURN NEW;
END;
$demon_modification_trigger$ LANGUAGE plpgsql;