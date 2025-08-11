-- Add down migration script here
CREATE OR REPLACE FUNCTION record_score(progress FLOAT, demon FLOAT, list_size FLOAT, requirement FLOAT) RETURNS FLOAT AS
$record_score$
SELECT CASE
           WHEN progress = 100 THEN
               CASE

                   WHEN 55 < demon AND demon <= 150 THEN
                       (56.191 * EXP(LN(2) * ((54.147 - (demon + 3.2)) * LN(50.0)) / 99.0)) + 6.273
                   WHEN 35 < demon AND demon <= 55 THEN
                       212.61 * (EXP(LN(1.036) * (1 - demon))) + 25.071
                   WHEN 20 < demon AND demon <= 35 THEN
                       (250 - 83.389) * (EXP(LN(1.0099685) * (2 - demon))) - 31.152
                   WHEN demon <= 20 THEN
                       (250 - 100.39) * (EXP(LN(1.168) * (1 - demon))) + 100.39

                   END

           WHEN progress < requirement THEN
               0.0
           ELSE
               CASE

                   WHEN 55 < demon AND demon <= 150 THEN
                       ((56.191 * EXP(LN(2) * ((54.147 - (demon + 3.2)) * LN(50.0)) / 99.0)) + 6.273) * (EXP(LN(5) * (progress - requirement) / (100 - requirement))) / 10
                   WHEN 35 < demon AND demon <= 55 THEN
                       (212.61 * (EXP(LN(1.036) * (1 - demon))) + 25.071) * (EXP(LN(5) * (progress - requirement) / (100 - requirement))) / 10
                   WHEN 20 < demon AND demon <= 35 THEN
                       ((250 - 83.389) * (EXP(LN(1.0099685) * (2 - demon))) - 31.152) * (EXP(LN(5) * (progress - requirement) / (100 - requirement))) / 10
                   WHEN demon <= 20 THEN
                       ((250 - 100.39) * (EXP(LN(1.168) * (1 - demon))) + 100.39) * (EXP(LN(5) * (progress - requirement) / (100 - requirement))) / 10

                   END
           END;
$record_score$
LANGUAGE SQL IMMUTABLE;

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

CREATE OR REPLACE VIEW ranked_players AS
SELECT
    ROW_NUMBER() OVER(ORDER BY players.score DESC, id) AS index,
    RANK() OVER(ORDER BY players.score DESC) AS rank,
    id, name, players.score, subdivision,
    nationalities.iso_country_code,
    nationalities.nation,
    nationalities.continent
FROM players
         LEFT OUTER JOIN nationalities
                         ON players.nationality = nationalities.iso_country_code
WHERE NOT players.banned AND players.score > 0.0;

SELECT recompute_player_scores();

ALTER TABLE players
    ALTER COLUMN score
    SET NOT NULL;