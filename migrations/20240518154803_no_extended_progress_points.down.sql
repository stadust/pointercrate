-- Add down migration script here

CREATE OR REPLACE VIEW score_giving AS
    SELECT records.progress, demons.position, demons.requirement, records.player
    FROM records
    INNER JOIN demons
    ON demons.id = records.demon
    WHERE records.status_ = 'APPROVED'

    UNION

    SELECT 100, demons.position, demons.requirement, demons.verifier
    FROM demons;

SELECT recompute_player_scores();
SELECT recompute_nation_scores();
SELECT recompute_subdivision_scores();