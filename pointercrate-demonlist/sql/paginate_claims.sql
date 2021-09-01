SELECT player_claims.id, members.member_id AS mid, members.name AS mname, players.id AS pid, players.name::TEXT as pname, verified
FROM player_claims
    INNER JOIN members on members.member_id=player_claims.member_id
    INNER JOIN players on players.id=player_id
WHERE (player_claims.id < $1 OR $1 IS NULL)
  AND (player_claims.id > $2 OR $2 IS NULL)
  AND ((STRPOS(players.name, $3::CITEXT) > 0 OR $3 is NULL)
  OR (STRPOS(members.name::CITEXT, $3::CITEXT) > 0 OR $3 is NULL))
  AND (verified = $4 OR $4 IS NULL)
ORDER BY id {}
LIMIT $5