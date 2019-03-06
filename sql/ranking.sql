SELECT RANK() OVER(ORDER BY t.total_score DESC) as rank, t.total_score as score, players.name as name, t.player as id
FROM
(
    SELECT a.player, SUM(record_score(a.progress::FLOAT, a.position::FLOAT, (SELECT {0} FROM aux))) as total_score
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
        WHERE demons.position <= {0} AND NOT players.banned
    ) a
    GROUP BY player
) t
INNER JOIN players
ON t.player = players.id
WHERE t.total_score >= record_score(100.0::FLOAT, {0}::FLOAT, (SELECT {0} FROM aux));
