-- Add up migration script here

ALTER TABLE players ADD COLUMN score DOUBLE PRECISION DEFAULT 0 NOT NULL;

CREATE VIEW score_giving AS
    SELECT records.progress, demons.position, demons.requirement, records.player
    FROM records
    INNER JOIN demons
    ON demons.id = records.demon
    WHERE records.status_ = 'APPROVED'

    UNION

    SELECT 100, demons.position, demons.requirement, demons.verifier
    FROM demons;

CREATE FUNCTION score_of_player(player_id INTEGER) RETURNS DOUBLE PRECISION AS $$
    SELECT SUM(record_score(progress, position, 150, requirement)) 
    FROM score_giving
    WHERE player = player_id
$$ LANGUAGE SQL;

-- This is slower than the old "select * from players_with_scores", but only needs to be called
-- when demons are being moved around, so overall cheaper. Should this ever become a bottleneck for
-- some obscure reason, we can separate the "score" column into a separate table, in which case the
-- below function becomes a "TRUNCATE + INSERT q" on that new table (but then we'd pay the cost of 
-- combining these via a JOIN when requesting the stats viewer).
CREATE FUNCTION recompute_player_scores() RETURNS void AS $$ 
    -- The nested query is faster than the more obvious "UPDATE players SET score = score_of_player(id)",
    -- as the latter would essentially have runtime O(|records| * |players|), which this solution as
    -- runtime O(|records| + |players|^2) [approximately, technically its |players| * |players where score > 0| 
    -- and I'm sure the query planner is clever enough to not make it quadratic].
    -- Since |records| >> |players|, this is faster.
    UPDATE players 
    SET score = coalesce(q.score, 0)
    FROM (
        SELECT player, SUM(record_score(progress, position, 150, requirement)) as score
        FROM score_giving
        GROUP BY player
    ) q
    WHERE q.player = id;
$$ LANGUAGE SQL;

SELECT recompute_player_scores();

DROP VIEW players_with_score;
CREATE VIEW ranked_players AS 
    SELECT 
        ROW_NUMBER() OVER(ORDER BY players.score DESC, id) AS index,
        RANK() OVER(ORDER BY score DESC) AS rank,
        id, name, players.score, subdivision,
        nationalities.iso_country_code,
        nationalities.nation,
        nationalities.continent
    FROM players
    LEFT OUTER JOIN nationalities
                 ON players.nationality = nationalities.iso_country_code
    WHERE NOT players.banned AND players.score > 0.0;


ALTER TABLE nationalities ADD COLUMN score DOUBLE PRECISION NOT NULL DEFAULT 0.0;

CREATE FUNCTION score_of_nation(iso_country_code VARCHAR(2)) RETURNS DOUBLE PRECISION AS $$
    SELECT SUM(record_score(q.progress, q.position, 150, q.requirement))
    FROM (
        SELECT DISTINCT ON (position) * from score_giving
        INNER JOIN players 
                ON players.id=player
        WHERE players.nationality = iso_country_code
        ORDER BY position, progress DESC
    ) q
$$ LANGUAGE SQL;

CREATE FUNCTION recompute_nation_scores() RETURNS void AS $$
    UPDATE nationalities
    SET score = COALESCE(p.sum, 0)
    FROM (
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
    WHERE p.nationality = iso_country_code
$$ LANGUAGE SQL;

SELECT recompute_nation_scores();

ALTER TABLE subdivisions ADD COLUMN score DOUBLE PRECISION NOT NULL DEFAULT 0.0;

CREATE FUNCTION score_of_subdivision(iso_country_code VARCHAR(2), iso_code VARCHAR(3)) RETURNS DOUBLE PRECISION AS $$
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

CREATE FUNCTION recompute_subdivision_scores() RETURNS void AS $$
    UPDATE subdivisions
    SET score = COALESCE(p.sum, 0)
    FROM (
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
    WHERE p.nationality = nation
      AND p.subdivision = iso_code
$$ LANGUAGE SQL;

SELECT recompute_subdivision_scores();

DROP VIEW nations_with_score;
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

-- Now-unused functions
DROP FUNCTION subdivision_ranking_of(country varchar(2));
DROP FUNCTION best_records_local(country VARCHAR(2), the_subdivision VARCHAR(3));

-- Hardening against invalid database state
ALTER TABLE players ADD CONSTRAINT nation_subdivions_fkey FOREIGN KEY (nationality, subdivision) REFERENCES subdivisions (nation, iso_code);