-- Your SQL goes here

ALTER TABLE demons ADD COLUMN thumbnail TEXT NOT NULL DEFAULT 'https://i.ytimg.com/vi/zebrafishes/mqdefault.jpg';

UPDATE demons
SET thumbnail = 'https://i.ytimg.com/vi/' || SUBSTRING(video FROM '%v=#"___________#"%' FOR '#') || '/mqdefault.jpg'
WHERE video IS NOT NULL
  AND NOT EXISTS (SELECT 1 FROM players WHERE players.id=demons.verifier AND players.link_banned);

CREATE OR REPLACE FUNCTION set_initial_thumbnail() RETURNS trigger AS '
BEGIN
    IF NEW.video IS NOT NULL AND NOT EXISTS(SELECT 1 FROM players WHERE players.id=NEW.verifier AND players.link_banned) THEN
        NEW.thumbnail := ''https://i.ytimg.com/vi/'' || SUBSTRING(NEW.video FROM ''%v=#"___________#"%'' FOR ''#'') || ''/mqdefault.jpg'';
    END IF;
    RETURN NEW;
END;
' LANGUAGE plpgsql;

CREATE TRIGGER demons_insert_set_thumbnail BEFORE INSERT ON demons FOR
EACH ROW EXECUTE PROCEDURE set_initial_thumbnail();

-- Your SQL goes here

ALTER TABLE demon_modifications ADD COLUMN thumbnail TEXT NULL DEFAULT NULL;

CREATE OR REPLACE FUNCTION audit_demon_modification() RETURNS trigger AS $demon_modification_trigger$
DECLARE
    name_change CITEXT;
    position_change SMALLINT;
    requirement_change SMALLINT;
    video_change VARCHAR(200);
    thumbnail_change TEXT;
    verifier_change INT;
    publisher_change INT;
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

    INSERT INTO demon_modifications (userid, name, position, requirement, video, verifier, publisher, thumbnail, id)
        (SELECT id, name_change, position_change, requirement_change, video_change, verifier_change, publisher_change, thumbnail_change, NEW.id
         FROM active_user LIMIT 1);

    RETURN NEW;
END;
$demon_modification_trigger$ LANGUAGE plpgsql;

DROP FUNCTION list_at(TIMESTAMP WITHOUT TIME ZONE);

CREATE FUNCTION list_at(TIMESTAMP WITHOUT TIME ZONE)
    RETURNS TABLE (
                      name CITEXT,
                      position_ SMALLINT,
                      requirement SMALLINT,
                      video VARCHAR(200),
                      thumbnail TEXT,
                      verifier INTEGER,
                      publisher INTEGER,
                      id INTEGER,
                      level_id BIGINT,
                      current_position SMALLINT
                  )
AS $$
SELECT name, CASE WHEN t.position IS NULL THEN demons.position ELSE t.position END, requirement, video, thumbnail, verifier, publisher, demons.id, level_id, demons.position AS current_position
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