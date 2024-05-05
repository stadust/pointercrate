-- Add up migration script here

ALTER TABLE players ADD COLUMN score DOUBLE PRECISION DEFAULT 0 NOT NULL;

CREATE VIEW score_giving AS
    SELECT records.progress, demons.position, demons.requirement, records.player
    FROM records
    INNER JOIN demons
    ON demons.id = records.demon
    WHERE records.status_ = 'APPROVED'

    UNION

    SELECT 100, demons.position, demons.requirement, demons.verifier
    FROM demons;

CREATE FUNCTION score_of_player(player_id INTEGER) RETURNS DOUBLE PRECISION AS $$
    SELECT SUM(record_score(progress, position, 150, requirement)) 
    FROM score_giving
    WHERE player = player_id
$$ LANGUAGE SQL;

-- This is slower than the old "select * from players_with_scores", but only needs to be called
-- when demons are being moved around, so overall cheaper. Should this ever become a bottleneck for
-- some obscure reason, we can separate the "score" column into a separate table, in which case the
-- below function becomes a "TRUNCATE + INSERT q" on that new table (but then we'd pay the cost of 
-- combining these via a JOIN when requesting the stats viewer).
CREATE FUNCTION recompute_player_scores() RETURNS void AS $$ 
    -- The nested query is faster than the more obvious "UPDATE players SET score = score_of_player(id)",
    -- as the latter would essentially have runtime O(|records| * |players|), which this solution as
    -- runtime O(|records| + |players|^2) [approximately, technically its |players| * |players where score > 0| 
    -- and I'm sure the query planner is clever enough to not make it quadratic].
    -- Since |records| >> |players|, this is faster.
    UPDATE players 
    SET score = coalesce(q.score, 0)
    FROM (
        SELECT player, SUM(record_score(progress, position, 150, requirement)) as score
        FROM score_giving
        GROUP BY player
    ) q
    WHERE q.player = id;
$$ LANGUAGE SQL;

SELECT recompute_player_scores();

DROP VIEW players_with_score;
CREATE VIEW ranked_players AS 
    SELECT 
        ROW_NUMBER() OVER(ORDER BY score DESC, id) AS index,
        RANK() OVER(ORDER BY score DESC) AS rank,
        id, name, score, subdivision,
        nationalities.iso_country_code,
        nationalities.nation,
        nationalities.continent
    FROM players
    LEFT OUTER JOIN nationalities
                 ON players.nationality = nationalities.iso_country_code
    WHERE NOT players.banned AND players.score > 0.0;
