DROP VIEW score_giving;

CREATE VIEW score_giving AS
    SELECT records.progress, demons.position, demons.requirement, records.player
    FROM records
    INNER JOIN demons
    ON demons.id = records.demon
    WHERE records.status_ = 'APPROVED' AND (demons.position <= 75 OR records.progress = 100)

    UNION

    SELECT 100, demons.position, demons.requirement, demons.verifier
    FROM demons;

DROP FUNCTION score_of_player(BOOLEAN, INTEGER);
CREATE OR REPLACE FUNCTION score_of_player(player_id INTEGER) RETURNS DOUBLE PRECISION AS $$
    SELECT SUM(record_score(progress, position, 150, requirement)) 
    FROM score_giving
    WHERE player = player_id
$$ LANGUAGE SQL;

CREATE OR REPLACE FUNCTION recompute_player_scores() RETURNS void AS $$ 
    UPDATE players 
    SET score = coalesce(q.score, 0)
    FROM players p
        LEFT OUTER JOIN (
            SELECT player, SUM(record_score(progress, position, 150, requirement)) as score
            FROM score_giving
            GROUP BY player
        ) q
        ON q.player = p.id
    WHERE players.id = p.id;
$$ LANGUAGE SQL;

DROP FUNCTION score_of_nation(BOOLEAN, VARCHAR(2));
CREATE OR REPLACE FUNCTION score_of_nation(iso_country_code VARCHAR(2)) RETURNS DOUBLE PRECISION AS $$
    SELECT SUM(record_score(q.progress, q.position, 150, q.requirement))
    FROM (
        SELECT DISTINCT ON (position) * from score_giving
        INNER JOIN players 
                ON players.id=player
        WHERE players.nationality = iso_country_code
        ORDER BY position, progress DESC
    ) q
$$ LANGUAGE SQL;

CREATE OR REPLACE FUNCTION recompute_nation_scores() RETURNS void AS $$
    UPDATE nationalities
    SET score = COALESCE(p.sum, 0)
    FROM nationalities n 
        LEFT OUTER JOIN (
            SELECT nationality, SUM(record_score(q.progress, q.position, 150, q.requirement))
            FROM (
                SELECT DISTINCT ON (position, nationality) * from score_giving
                INNER JOIN players 
                        ON players.id=player
                WHERE players.nationality IS NOT NULL
                ORDER BY players.nationality, position, progress DESC
            ) q
            GROUP BY nationality
        ) p
        ON p.nationality = n.iso_country_code
    WHERE n.iso_country_code = nationalities.iso_country_code
$$ LANGUAGE SQL;

DROP FUNCTION score_of_subdivision(BOOLEAN, VARCHAR(2), VARCHAR(3));
CREATE OR REPLACE FUNCTION score_of_subdivision(iso_country_code VARCHAR(2), iso_code VARCHAR(3)) RETURNS DOUBLE PRECISION AS $$
    SELECT SUM(record_score(q.progress, q.position, 150, q.requirement))
    FROM (
        SELECT DISTINCT ON (position) * from score_giving
        INNER JOIN players 
                ON players.id=player
        WHERE players.nationality = iso_country_code
          AND players.subdivision = iso_code
        ORDER BY position, progress DESC
    ) q
$$ LANGUAGE SQL;

CREATE OR REPLACE FUNCTION recompute_subdivision_scores() RETURNS void AS $$
    UPDATE subdivisions
    SET score = COALESCE(p.sum, 0)
    FROM subdivisions s 
        LEFT OUTER JOIN (
            SELECT nationality, subdivision, SUM(record_score(q.progress, q.position, 150, q.requirement))
            FROM (
                SELECT DISTINCT ON (position, nationality, subdivision) * from score_giving
                INNER JOIN players 
                        ON players.id=player
                WHERE players.nationality IS NOT NULL
                AND players.subdivision IS NOT NULL
                ORDER BY players.nationality, players.subdivision, position, progress DESC
            ) q
            GROUP BY nationality, subdivision
        ) p
        ON s.nation = p.nationality AND s.iso_code = p.subdivision
    WHERE s.nation = subdivisions.nation
      AND s.iso_code = subdivisions.iso_code
$$ LANGUAGE SQL;

DROP VIEW ranked_players;
DROP MATERIALIZED VIEW player_ranks;

CREATE MATERIALIZED VIEW player_ranks AS
       SELECT
           RANK() OVER (ORDER BY score DESC) as rank,
           id
       FROM players
       WHERE
           score != 0 AND NOT banned;

CREATE UNIQUE INDEX player_ranks_id_idx ON player_ranks(id);

CREATE VIEW ranked_players AS
SELECT
    ROW_NUMBER() OVER(ORDER BY rank, id) AS index,
    rank,
    id, name, players.score, subdivision,
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
        ROW_NUMBER() OVER(ORDER BY score DESC, iso_country_code) AS index,
        RANK() OVER(ORDER BY score DESC) AS rank,
        score,
        iso_country_code,
        nation,
        continent
    FROM nationalities
    WHERE score > 0.0;

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

SELECT recompute_player_scores();
SELECT recompute_nation_scores();
SELECT recompute_subdivision_scores();

ALTER TABLE demons DROP COLUMN rated_position;
DROP FUNCTION recompute_rated_positions();

ALTER TABLE demons DROP COLUMN rated;
ALTER TABLE players DROP COLUMN ratedplus_score;
ALTER TABLE nationalities DROP COLUMN ratedplus_score;
ALTER TABLE subdivisions DROP COLUMN ratedplus_score;

ALTER TABLE demon_modifications DROP COLUMN rated;
ALTER TABLE demon_modifications DROP COLUMN rated_position;