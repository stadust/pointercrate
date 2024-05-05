-- Add down migration script here

ALTER TABLE players DROP COLUMN score;

DROP FUNCTION recompute_all_scores();
DROP FUNCTION score_of_player(player_id INTEGER);
DROP VIEW score_giving;