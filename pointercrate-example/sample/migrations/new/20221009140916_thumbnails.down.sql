-- This file should undo anything in `up.sql`

DROP TRIGGER demons_insert_set_thumbnail ON demons;
DROP FUNCTION set_initial_thumbnail;

ALTER TABLE demons DROP COLUMN thumbnail;

CREATE OR REPLACE FUNCTION audit_demon_modification() RETURNS trigger AS $demon_modification_trigger$
DECLARE
    name_change CITEXT;
    position_change SMALLINT;
    requirement_change SMALLINT;
    video_change VARCHAR(200);
    verifier_change INT;
    publisher_change InT;
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

ALTER TABLE demon_modifications DROP COLUMN thumbnail;

DROP FUNCTION list_at(TIMESTAMP WITHOUT TIME ZONE);
CREATE OR REPLACE FUNCTION list_at(TIMESTAMP WITHOUT TIME ZONE)
    RETURNS TABLE (
                      name CITEXT,
                      position_ SMALLINT,
                      requirement SMALLINT,
                      video VARCHAR(200),
                      verifier INTEGER,
                      publisher INTEGER,
                      id INTEGER,
                      level_id BIGINT,
                      current_position SMALLINT
                  )
AS $$
SELECT name, CASE WHEN t.position IS NULL THEN demons.position ELSE t.position END, requirement, video, verifier, publisher, demons.id, level_id, demons.position AS current_position
FROM demons
         LEFT OUTER JOIN (
    SELECT DISTINCT ON (id) id, position
    FROM demon_modifications
    WHERE time >= $1 AND position != -1
    ORDER BY id, time
) t
                         ON demons.id = t.id
WHERE NOT EXISTS (SELECT 1 FROM demon_additions WHERE demon_additions.id = demons.id AND time >= $1)
$$
    LANGUAGE SQL
    STABLE;