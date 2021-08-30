SELECT player_claims.id, members.member_id AS mid, members.name AS mname, players.id AS pid, players.name::TEXT as pname, verified
FROM player_claims
    INNER JOIN members on members.member_id=player_claims.member_id
    INNER JOIN players on players.id=player_id
WHERE (player_claims.id < $1 OR $1 IS NULL)
  AND (player_claims.id > $2 OR $2 IS NULL)
  AND (players.name = $3 OR $3 IS NULL)
  AND (members.name = $4 OR $4 IS NULL)
ORDER BY id {}