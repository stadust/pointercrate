-- Add down migration script here
DROP VIEW ranked_players;

ALTER TABLE players DROP COLUMN score;
ALTER TABLE nationalities DROP COLUMN score;
ALTER TABLE subdivisions DROP COLUMN score;

DROP FUNCTION recompute_player_scores();
DROP FUNCTION score_of_player(player_id INTEGER);
DROP FUNCTION recompute_nation_scores();
DROP FUNCTION score_of_nation(iso_country_code VARCHAR(2));
DROP FUNCTION recompute_subdivision_scores();
DROP FUNCTION score_of_subdivision(iso_country_code VARCHAR(2), iso_code VARCHAR(3));
DROP VIEW score_giving;

-- Copied from 20210419002933.up
CREATE VIEW players_with_score AS
SELECT players.id,
       players.name,
       RANK() OVER(ORDER BY scores.total_score DESC) AS rank,
       CASE WHEN scores.total_score IS NULL THEN 0.0::FLOAT ELSE scores.total_score END AS score,
       ROW_NUMBER() OVER(ORDER BY scores.total_score DESC) AS index,
       nationalities.iso_country_code,
       nationalities.nation,
       players.subdivision,
       nationalities.continent
FROM
    (
        SELECT pseudo_records.player,
               SUM(record_score(pseudo_records.progress::FLOAT, pseudo_records.position::FLOAT, 100::FLOAT, pseudo_records.requirement)) as total_score
        FROM (
                 SELECT player,
                        progress,
                        position,
                        CASE WHEN demons.position > 75 THEN 100 ELSE requirement END AS requirement
                 FROM records
                          INNER JOIN demons
                                     ON demons.id = demon
                 WHERE demons.position <= 150 AND status_ = 'APPROVED' AND (demons.position <= 75 OR progress = 100)

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
WHERE NOT players.banned AND players.id != 1534;