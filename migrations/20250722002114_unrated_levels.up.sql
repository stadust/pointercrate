ALTER TABLE demons ADD COLUMN rated BOOLEAN NOT NULL DEFAULT TRUE;
ALTER TABLE players ADD COLUMN ratedplus_score DOUBLE PRECISION NOT NULL DEFAULT 0.0;
ALTER TABLE nationalities ADD COLUMN ratedplus_score DOUBLE PRECISION NOT NULL DEFAULT 0.0;
ALTER TABLE subdivisions ADD COLUMN ratedplus_score DOUBLE PRECISION NOT NULL DEFAULT 0.0;

ALTER TABLE demons ADD COLUMN rated_position SMALLINT DEFAULT NULL;
UPDATE demons SET rated_position = position;

ALTER TABLE demons ADD CONSTRAINT unique_rated_position UNIQUE (rated_position) DEFERRABLE INITIALLY DEFERRED;

CREATE OR REPLACE FUNCTION recompute_rated_positions() RETURNS void AS $$
    BEGIN
        UPDATE demons SET rated_position = NULL WHERE NOT rated;

        WITH filtered AS (
            SELECT id, ROW_NUMBER() OVER (ORDER BY demons.position) AS rated_position
            FROM demons
            WHERE rated
        )
        UPDATE demons
        SET rated_position = filtered.rated_position
        FROM filtered
        WHERE demons.id = filtered.id;
    END;
$$ LANGUAGE plpgsql;

SELECT recompute_rated_positions();

-- stats viewer stuff
DROP VIEW score_giving;

CREATE VIEW score_giving AS
    SELECT records.progress, demons.position, demons.requirement, records.player, FALSE AS rated_list
    FROM records
    INNER JOIN demons
    ON demons.id = records.demon
    WHERE records.status_ = 'APPROVED' AND (demons.position <= 75 OR demons.rated_position <= 75 OR records.progress = 100)

	UNION

	SELECT records.progress, demons.rated_position, demons.requirement, records.player, TRUE AS rated_list
    FROM records
    INNER JOIN demons
    ON demons.id = records.demon
    WHERE demons.rated = TRUE AND records.status_ = 'APPROVED' AND (demons.position <= 75 OR demons.rated_position <= 75 OR records.progress = 100) 

    UNION

    SELECT 100, demons.position, demons.requirement, demons.verifier, FALSE AS rated_list
    FROM demons

	UNION
	
	SELECT 100, demons.rated_position, demons.requirement, demons.verifier, TRUE AS rated_list
    FROM demons
	WHERE demons.rated = TRUE;

DROP FUNCTION score_of_player(INTEGER);
CREATE OR REPLACE FUNCTION score_of_player(is_rated BOOLEAN, player_id INTEGER) RETURNS DOUBLE PRECISION AS $$
    SELECT SUM(record_score(progress, position, 150, requirement)) 
    FROM score_giving
    WHERE player = player_id
    AND rated_list = is_rated
$$ LANGUAGE SQL;

CREATE OR REPLACE FUNCTION recompute_player_scores() RETURNS void AS $$
    UPDATE players
    SET score = COALESCE(q.score, 0), ratedplus_score = COALESCE(q.ratedplus_score, 0)
    FROM (
        SELECT player, 
        SUM(record_score(progress, position, 150, requirement))
            FILTER (WHERE NOT rated_list) AS ratedplus_score,
        SUM(record_score(progress, position, 150, requirement))
            FILTER (WHERE rated_list) AS score
        FROM score_giving
        GROUP BY player
    ) q
    WHERE q.player = id;
$$ LANGUAGE SQL;

DROP FUNCTION score_of_nation(VARCHAR(2));
CREATE OR REPLACE FUNCTION score_of_nation(is_rated BOOLEAN, iso_country_code VARCHAR(2)) RETURNS DOUBLE PRECISION AS $$
    SELECT SUM(record_score(q.progress, q.position, 150, q.requirement))
    FROM (
        SELECT DISTINCT ON (position) * from score_giving
        INNER JOIN players 
                ON players.id=player
        WHERE players.nationality = iso_country_code AND rated_list = is_rated
        ORDER BY position, progress DESC
    ) q
$$ LANGUAGE SQL;

CREATE OR REPLACE FUNCTION recompute_nation_scores() RETURNS void AS $$
    UPDATE nationalities
    SET score = COALESCE(p.score, 0), ratedplus_score = COALESCE(p.ratedplus_score, 0)
    FROM (
        SELECT nationality,
        SUM(record_score(q.progress, q.position, 150, q.requirement))
            FILTER (WHERE q.rated_list) AS score,
        SUM(record_score(q.progress, q.position, 150, q.requirement))
            FILTER (WHERE NOT q.rated_list) AS ratedplus_score
        FROM (
            SELECT DISTINCT ON (position, nationality, rated_list) * from score_giving
            INNER JOIN players 
                    ON players.id = player
            WHERE players.nationality IS NOT NULL
            ORDER BY players.nationality, position, rated_list, progress DESC
        ) q
        GROUP BY nationality
    ) p
    WHERE p.nationality = iso_country_code
$$ LANGUAGE SQL;

DROP FUNCTION score_of_subdivision(VARCHAR(2), VARCHAR(3));
CREATE OR REPLACE FUNCTION score_of_subdivision(is_rated BOOLEAN, iso_country_code VARCHAR(2), iso_code VARCHAR(3)) RETURNS DOUBLE PRECISION AS $$
    SELECT SUM(record_score(q.progress, q.position, 150, q.requirement))
    FROM (
        SELECT DISTINCT ON (position) * from score_giving
        INNER JOIN players 
                ON players.id=player
        WHERE players.nationality = iso_country_code
          AND players.subdivision = iso_code
          AND rated_list = is_rated
        ORDER BY position, progress DESC
    ) q
$$ LANGUAGE SQL;

CREATE OR REPLACE FUNCTION recompute_subdivision_scores() RETURNS void AS $$
    UPDATE subdivisions
    SET score = COALESCE(p.score, 0), ratedplus_score = COALESCE(p.ratedplus_score, 0)
    FROM (
        SELECT nationality, subdivision,
            SUM(record_score(q.progress, q.position, 150, q.requirement))
                FILTER (WHERE q.rated_list) AS score,
            SUM(record_score(q.progress, q.position, 150, q.requirement))
                FILTER (WHERE NOT q.rated_list) AS ratedplus_score
        FROM (
            SELECT DISTINCT ON (position, nationality, subdivision, rated_list) * from score_giving
            INNER JOIN players 
                    ON players.id=player
            WHERE players.nationality IS NOT NULL
              AND players.subdivision IS NOT NULL
            ORDER BY players.nationality, players.subdivision, position, rated_list, progress DESC
        ) q
        GROUP BY nationality, subdivision
    ) p
    WHERE p.nationality = nation
      AND p.subdivision = iso_code
$$ LANGUAGE SQL;

SELECT recompute_player_scores();
SELECT recompute_nation_scores();
SELECT recompute_subdivision_scores();

DROP VIEW ranked_players;
DROP MATERIALIZED VIEW player_ranks;

CREATE MATERIALIZED VIEW player_ranks AS
SELECT
    CASE WHEN score != 0 THEN RANK() OVER (ORDER BY score DESC) END AS rank,
    CASE WHEN ratedplus_score != 0 THEN RANK() OVER (ORDER BY ratedplus_score DESC) END AS ratedplus_rank,
    id
FROM players
WHERE ratedplus_score != 0 OR score != 0 AND NOT banned;

CREATE UNIQUE INDEX player_ranks_id_idx ON player_ranks(id);

CREATE VIEW ranked_players AS
SELECT
    ROW_NUMBER() OVER(ORDER BY rank, id) AS index,
    ROW_NUMBER() OVER (ORDER BY ratedplus_rank, id) AS ratedplus_index,
    rank,
    ratedplus_rank,
    id, name, players.score, players.ratedplus_score,
    subdivision,
    nationalities.iso_country_code,
    nationalities.nation,
    nationalities.continent
FROM players
LEFT OUTER JOIN nationalities
    ON players.nationality = nationalities.iso_country_code
NATURAL JOIN player_ranks;

DROP VIEW ranked_nations;

CREATE VIEW ranked_nations AS 
    SELECT 
        ROW_NUMBER() OVER (ORDER BY score DESC, iso_country_code) AS index,
        ROW_NUMBER() OVER (ORDER BY ratedplus_score DESC, iso_country_code) AS ratedplus_index,
        CASE WHEN score != 0 THEN RANK() OVER (ORDER BY score DESC) END AS rank,
        CASE WHEN ratedplus_score != 0 THEN RANK() OVER (ORDER BY ratedplus_score DESC) END AS ratedplus_rank,
        score,
        ratedplus_score,
        iso_country_code,
        nation,
        continent
    FROM nationalities
    WHERE score > 0.0 OR ratedplus_score > 0.0;

-- audit log stuff
ALTER TABLE demon_modifications ADD COLUMN rated BOOLEAN NULL DEFAULT NULL;
ALTER TABLE demon_modifications ADD COLUMN rated_position SMALLINT NULL DEFAULT NULL;

UPDATE demon_modifications SET rated_position = position WHERE position != -1;

CREATE OR REPLACE FUNCTION audit_demon_modification() RETURNS trigger AS $demon_modification_trigger$
DECLARE
    name_change CITEXT;
    position_change SMALLINT;
    rated_position_change SMALLINT;
    requirement_change SMALLINT;
    video_change VARCHAR(200);
    thumbnail_change TEXT;
    verifier_change INT;
    publisher_change INT;
    rated_change BOOLEAN;
BEGIN
    IF (OLD.name <> NEW.name) THEN
        name_change = OLD.name;
    END IF;

    IF (OLD.position <> NEW.position) THEN
        position_change = OLD.position;
    END IF;

    IF (OLD.rated_position IS DISTINCT FROM NEW.rated_position) THEN
        rated_position_change = OLD.rated_position;
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

    IF (OLD.rated <> NEW.rated) THEN
        rated_change = OLD.rated;
    END IF;

    INSERT INTO demon_modifications (userid, name, position, rated_position, requirement, video, verifier, publisher, thumbnail, rated, id)
        (SELECT id, name_change, position_change, rated_position_change, requirement_change, video_change, verifier_change, publisher_change, thumbnail_change, rated_change, NEW.id
         FROM active_user LIMIT 1);

    RETURN NEW;
END;
$demon_modification_trigger$ LANGUAGE plpgsql;
