SELECT records.id, progress, records.video::text, status_::text AS status,
       players.id AS player_id, players.name::text AS player_name, players.banned AS player_banned,
       demons.id AS demon_id, demons.name::text AS demon_name, demons.position,
FROM records
INNER JOIN players ON records.player = players.id
INNER JOIN demons ON records.demon = demons.id
WHERE (record_id < $1 OR $1 IS NULL)
  AND (record_id > $2 OR $2 IS NULL)
  AND (progress = $3 OR $3 IS NULL)
  AND (progress < $4 OR $4 IS NULL)
  AND (progress > $5 OR $5 IS NULL)
  AND (position = $6 OR $6 IS NULL)
  AND (position < $7 OR $7 IS NULL)
  AND (position > $8 OR $8 IS NULL)
  AND (status_ = CAST($9::TEXT AS record_status) OR $9 IS NULL)
  AND (demon_name = $10 OR $10 IS NULL)
  AND (demon_id = $11 OR $11 IS NULL)
  AND (video = $12 OR (video IS NULL AND $13) OR ($12 IS NULL  AND NOT $13))
LIMIT $14
ORDER BY id ASC