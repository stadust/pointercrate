-- Your SQL goes here

CREATE VIEW players_with_score AS
    SELECT players.id as id,
           players.name as name,
           players.banned as banned,
           CASE WHEN t.total_score IS NULL THEN 0.0::FLOAT ELSE t.total_score END AS score,
           players.nationality as nationality
    FROM
    (
        SELECT a.player, SUM(record_score(a.progress::FLOAT, a.position::FLOAT, 100::FLOAT)) as total_score
        FROM (
            SELECT player as player, progress as progress, demons.position as position
            FROM records
            INNER JOIN demons
            ON demons.name = demon
            WHERE demons.position <= 100 AND status_ = 'APPROVED'
            UNION
            SELECT verifier as player, 100 as progress, position as position
            FROM demons
            INNER JOIN players
            ON players.id = verifier
            WHERE demons.position <= 100 AND NOT players.banned
        ) a
        GROUP BY player
    ) t
    RIGHT OUTER JOIN players
    ON t.player = players.id;