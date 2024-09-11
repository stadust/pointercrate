-- Add up migration script here

-- We need LEFT OUTER JOINs below so that those players which DO NOT show up in the 
-- SELECT player, SUM(...) query (because they no longer have any records that give scores) have their scores correctly
-- reset to 0! 

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

SELECT recompute_player_scores();
SELECT recompute_nation_scores();
SELECT recompute_subdivision_scores();