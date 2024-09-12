-- Your SQL goes here


-- I call this query Frank
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
               SUM(record_score(pseudo_records.progress::FLOAT, pseudo_records.position::FLOAT, 100::FLOAT)) as total_score
        FROM (
            SELECT player,
                   progress,
                   demons.position
            FROM records
            INNER JOIN demons
            ON demons.name = demon
            WHERE demons.position <= 100 AND status_ = 'APPROVED'

            UNION

            SELECT verifier as player,
                   CASE WHEN demons.position > 100 THEN 0.0::FLOAT ELSE 100.0::FLOAT END as progress,
                   position as position
            FROM demons

            UNION

            SELECT publisher as player,
                   0.0::FLOAT as progress,
                   position as position
            FROM demons

            UNION

            SELECT creator as player,
                   0.0::FLOAT as progress,
                   1.0::FLOAT as position -- doesn't matter
            FROM creators
        ) AS pseudo_records
        GROUP BY player
    ) scores
    INNER JOIN players
    ON scores.player = players.id
    LEFT OUTER JOIN nationalities
    ON players.nationality = nationalities.iso_country_code
    WHERE NOT players.banned;