-- Your SQL goes here

DROP VIEW players_with_score;
DROP FUNCTION record_score(FLOAT, FLOAT, FLOAT);


CREATE FUNCTION record_score(progress FLOAT, demon FLOAT, list_size FLOAT, requirement FLOAT) RETURNS FLOAT AS
$record_score$
    SELECT CASE
        WHEN progress = 100 THEN
            150.0 * EXP((1.0 - demon) * LN(1.0 / 30.0) / (-149.0))
        WHEN progress < requirement THEN
            0.0
        ELSE
            150.0 * EXP((1.0 - demon) * LN(1.0 / 30.0) / (-149.0)) * (0.25 * (progress - requirement) / (100 - requirement) + 0.25)
    END;
$record_score$
LANGUAGE SQL IMMUTABLE;

CREATE VIEW players_with_score AS
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
                   demons.position,
                   demons.requirement
            FROM records
            INNER JOIN demons
            ON demons.name = demon
            WHERE demons.position <= 150 AND status_ = 'APPROVED'

            UNION

            SELECT verifier as player,
                   CASE WHEN demons.position > 150 THEN 0.0::FLOAT ELSE 100.0::FLOAT END as progress,
                   position as position,
                   100.0::FLOAT
            FROM demons

            UNION

            SELECT publisher as player,
                   0.0::FLOAT as progress,
                   position as position,
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