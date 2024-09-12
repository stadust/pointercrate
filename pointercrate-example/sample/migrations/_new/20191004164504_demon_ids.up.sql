-- Your SQL goes here

-- Create the ID column
ALTER TABLE demons ADD COLUMN id SERIAL;

-- Fix up the audit logs
-- demon addition logs
ALTER TABLE demon_additions ADD COLUMN id INTEGER;

UPDATE demon_additions
SET id = demons.id
FROM demons
WHERE demon_additions.name = demons.name;

ALTER TABLE demon_additions ALTER COLUMN id SET NOT NULL;
ALTER TABLE demon_additions DROP COLUMN name;

CREATE OR REPLACE FUNCTION audit_demon_addition() RETURNS trigger AS $demon_add_trigger$
    BEGIN
        INSERT INTO demon_additions (userid, id) (SELECT id , NEW.id FROM active_user LIMIT 1);
        RETURN NEW;
    END;
$demon_add_trigger$ LANGUAGE plpgsql;

-- demon modification logs
ALTER TABLE demon_modifications ADD COLUMN id INTEGER;

UPDATE demon_modifications
SET id = demons.id
FROM demons
WHERE demon = demons.name;

ALTER TABLE demon_modifications ALTER COLUMN id SET NOT NULL;
ALTER TABLE demon_modifications DROP COLUMN demon;

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

-- record modifications
ALTER TABLE record_modifications RENAME COLUMN demon to demon_name;
ALTER TABLE record_modifications ADD COLUMN demon INTEGER;

UPDATE record_modifications
SET demon = demons.id
FROM demons
WHERE demon_name = demons.name;

ALTER TABLE record_modifications DROP COLUMN demon_name;

CREATE OR REPLACE FUNCTION audit_record_modification() RETURNS trigger AS $record_modification_trigger$
    DECLARE
        progress_change SMALLINT;
        video_change VARCHAR(200);
        status_change RECORD_STATUS;
        player_change INT;
        demon_change INTEGER;
    BEGIN
        if (OLD.progress <> NEW.progress) THEN
            progress_change = OLD.progress;
        END IF;

        IF (OLD.video <> NEW.video) THEN
            video_change = OLD.video;
        END IF;

        IF (OLD.status_ <> NEW.status_) THEN
            status_change = OLD.status_;
        END IF;

        IF (OLD.player <> NEW.player) THEN
            player_change = OLD.player;
        END IF;

        IF (OLD.demon <> NEW.demon) THEN
            demon_change = OLD.demon;
        END IF;

        INSERT INTO record_modifications (userid, id, progress, video, status_, player, demon)
            (SELECT id, NEW.id, progress_change, video_change, status_change, player_change, demon_change
            FROM active_user LIMIT 1);

        RETURN NEW;
    END;
$record_modification_trigger$ LANGUAGE plpgsql;

-- creator additions
ALTER TABLE creator_additions RENAME COLUMN demon TO demon_name;
ALTER TABLE creator_additions ADD COLUMN demon INTEGER;

UPDATE creator_additions
SET demon = demons.id
FROM demons
WHERE demon_name = demons.name;

ALTER TABLE creator_additions ALTER COLUMN demon SET NOT NULL;
ALTER TABLE creator_additions DROP COLUMN demon_name;

-- creator deletions
ALTER TABLE creator_deletions RENAME COLUMN demon TO demon_name;
ALTER TABLE creator_deletions ADD COLUMN demon INTEGER;

UPDATE creator_deletions
SET demon = demons.id
FROM demons
WHERE demon_name = demons.name;

ALTER TABLE creator_deletions ALTER COLUMN demon SET NOT NULL;
ALTER TABLE creator_deletions DROP COLUMN demon_name;

-- Fix up references from the creators table
ALTER TABLE creators RENAME COLUMN demon TO demon_name;
ALTER TABLE creators ADD COLUMN demon INTEGER;

-- No need to temporarily unregister triggers here, there is no 'creator_modifications' audit log
UPDATE creators
SET demon = demons.id
FROM demons
WHERE demon_name = demons.name;

ALTER TABLE creators DROP CONSTRAINT creators_pkey;
ALTER TABLE creators DROP COLUMN demon_name;

-- Fix up references from the records table
ALTER TABLE records RENAME COLUMN demon TO demon_name;
ALTER TABLE records ADD COLUMN demon INTEGER;

-- We need to temporarily unregister the trigger, otherwise this creates a ton of empty audit log entries
DROP TRIGGER record_modification_trigger ON records;

UPDATE records
SET demon = demons.id
FROM demons
WHERE demon_name = demons.name;

-- recreate trigger
CREATE TRIGGER record_modification_trigger AFTER UPDATE ON records FOR EACH ROW EXECUTE PROCEDURE audit_record_modification();

ALTER TABLE records ALTER COLUMN demon SET NOT NULL;

-- Fix up the views over the records table
DROP VIEW records_pds;
DROP VIEW records_pd;
CREATE VIEW records_pd AS  -- records with player and demon
    SELECT records.id, records.progress, records.video, records.status_, records.submitter AS submitter_id,
           players.id AS player_id, players.name AS player_name, players.banned AS player_banned,
           demons.id AS demon_id, demons.name AS demon_name, demons.position
    FROM records
    INNER JOIN players
    ON records.player = players.id
    INNER JOIN demons
    ON demons.id = records.demon;

CREATE VIEW records_pds AS  -- records with player, demon and submitter
    SELECT records_pd.id, records_pd.progress, records_pd.video, records_pd.status_,
           records_pd.player_id, records_pd.player_name, records_pd.player_banned,
           records_pd.demon_id, records_pd.demon_name, records_pd.position,
           submitters.submitter_id, submitters.banned AS submitter_banned
    FROM records_pd
    INNER JOIN submitters
    ON records_pd.submitter_id = submitters.submitter_id;

-- for minimal representation
DROP VIEW records_p;
CREATE VIEW records_p AS  -- records with player
    SELECT records.id, records.progress, records.video, records.status_, records.demon,
           players.id AS player_id, players.name AS player_name, players.banned AS player_banned
    FROM records
    INNER JOIN players
    ON records.player = players.id;

DROP VIEW records_d;
CREATE VIEW records_d AS  -- records with demon
    SELECT records.id, records.progress, records.video, records.status_, records.player,
           demons.id AS demon_id, demons.name AS demon_name, demons.position
    FROM records
    INNER JOIN demons
    ON demons.id = records.demon;

-- we need to re-declare the player ranking view since it referenced records.demon_name
CREATE OR REPLACE VIEW players_with_score AS
    SELECT players.id,
           players.name,
           RANK() OVER(ORDER BY scores.total_score DESC) AS rank,
           CASE WHEN scores.total_score IS NULL THEN 0.0::FLOAT ELSE scores.total_score END AS score,
           ROW_NUMBER() OVER(ORDER BY scores.total_score DESC) AS index,
           nationalities.iso_country_code,
           nationalities.nation
    FROM
    (
        SELECT pseudo_records.player,
               SUM(record_score(pseudo_records.progress::FLOAT, pseudo_records.position::FLOAT, 100::FLOAT, pseudo_records.requirement)) as total_score
        FROM (
            SELECT player,
                   progress,
                   position,
                   requirement
            FROM records
            INNER JOIN demons
            ON demons.id = demon
            WHERE demons.position <= 150 AND status_ = 'APPROVED'

            UNION

            SELECT verifier as player,
                   CASE WHEN demons.position > 150 THEN 0.0::FLOAT ELSE 100.0::FLOAT END as progress,
                   position,
                   100.0::FLOAT
            FROM demons

            UNION

            SELECT publisher as player,
                   0.0::FLOAT as progress,
                   position,
                   100.0::FLOAT
            FROM demons

            UNION

            SELECT creator as player,
                   0.0::FLOAT as progress,
                   1.0::FLOAT as position, -- doesn't matter
                   100.0::FLOAT
            FROM creators
        ) AS pseudo_records
        GROUP BY player
    ) scores
    INNER JOIN players
    ON scores.player = players.id
    LEFT OUTER JOIN nationalities
    ON players.nationality = nationalities.iso_country_code
    WHERE NOT players.banned;

-- replace the views of the demons table
DROP VIEW demons_pv;
CREATE VIEW demons_pv AS  -- demons with publisher and verifier
    SELECT demons.id, demons.position, demons.name, demons.requirement, demons.video,
           publishers.id AS publisher_id, publishers.name AS publisher_name, publishers.banned AS publisher_banned,
           verifiers.id AS verifier_id, verifiers.name AS verifier_name, verifiers.banned AS verifier_banned
    FROM demons
    INNER JOIN players AS verifiers
    ON verifiers.id = demons.verifier
    INNER JOIN players AS publishers
    ON publishers.id = demons.publisher;

DROP VIEW demons_p;
CREATE VIEW demons_p AS  -- demons with publisher
    SELECT demons.id, demons.position, demons.name, demons.video,
           publishers.id AS publisher_id, publishers.name AS publisher_name, publishers.banned AS publisher_banned
    FROM demons
    INNER JOIN players AS publishers
    ON publishers.id = demons.publisher;

-- Drop the old audit logs since we migrated the data over to the new ones already. Its a waste of time to update the references into the demons table
-- DROP TABLE audit_log;

-- Drop the old column
ALTER TABLE records DROP COLUMN demon_name;

-- change primary key on demons relation
ALTER TABLE demons DROP CONSTRAINT demons_pkey;
ALTER TABLE demons ADD PRIMARY KEY (id);

--set up new column to be foreign key to new primary key
ALTER TABLE creators ADD CONSTRAINT creators_demon_fkey FOREIGN KEY (demon) REFERENCES demons(id);
ALTER TABLE records ADD CONSTRAINT records_demon_fkey FOREIGN KEY (demon) REFERENCES demons(id);

-- set up the primary key of the creators table again
ALTER TABLE creators ADD CONSTRAINT creators_pkey PRIMARY KEY (demon, creator);
