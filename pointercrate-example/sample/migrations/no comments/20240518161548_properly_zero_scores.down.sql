-- Add down migration script here

CREATE OR REPLACE FUNCTION recompute_player_scores() RETURNS void AS $$ 
    UPDATE players 
    SET score = coalesce(q.score, 0)
    FROM (
        SELECT player, SUM(record_score(progress, position, 150, requirement)) as score
        FROM score_giving
        GROUP BY player
    ) q
    WHERE q.player = id;
$$ LANGUAGE SQL;

CREATE OR REPLACE FUNCTION recompute_nation_scores() RETURNS void AS $$
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

CREATE OR REPLACE FUNCTION recompute_subdivision_scores() RETURNS void AS $$
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


SELECT recompute_player_scores();
SELECT recompute_nation_scores();
SELECT recompute_subdivision_scores();