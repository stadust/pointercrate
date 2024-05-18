-- Add up migration script here
CREATE OR REPLACE VIEW score_giving AS
    SELECT records.progress, demons.position, demons.requirement, records.player
    FROM records
    INNER JOIN demons
    ON demons.id = records.demon
    WHERE records.status_ = 'APPROVED' AND (demons.position <= 75 OR records.progress = 100)

    UNION

    SELECT 100, demons.position, demons.requirement, demons.verifier
    FROM demons;


SELECT recompute_player_scores();
SELECT recompute_nation_scores();
SELECT recompute_subdivision_scores();