-- Your SQL goes here

CREATE FUNCTION record_score(progress FLOAT, demon FLOAT, list_size FLOAT) RETURNS FLOAT AS
$record_score$
    SELECT (progress / 100.0) ^ demon * list_size / (1.0 + (list_size - 1.0) * EXP(-4.0 * (list_size - demon) * LN(list_size - 1.0)/(3.0 * list_size)));
$record_score$
LANGUAGE SQL IMMUTABLE;