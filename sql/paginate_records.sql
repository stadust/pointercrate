SELECT records.id, progress, CASE WHEN players.link_banned THEN NULL ELSE records.video::text END, status_::text AS status,
       players.id AS player_id, players.name::text AS player_name, players.banned AS player_banned,
       demons.id AS demon_id, demons.name::text AS demon_name, demons.position
FROM records
INNER JOIN players ON records.player = players.id
INNER JOIN demons ON records.demon = demons.id
WHERE (records.id < $1 OR $1 IS NULL)
  AND (records.id > $2 OR $2 IS NULL)
  AND (progress = $3 OR $3 IS NULL)
  AND (progress < $4 OR $4 IS NULL)
  AND (progress > $5 OR $5 IS NULL)
  AND (position = $6 OR $6 IS NULL)
  AND (position < $7 OR $7 IS NULL)
  AND (position > $8 OR $8 IS NULL)
  AND (status_ = CAST($9::TEXT AS record_status) OR $9 IS NULL)
  AND (demons.name = $10::CITEXT OR $10 IS NULL)
  AND (demons.id = $11 OR $11 IS NULL)
  AND (records.video = $12 OR (records.video IS NULL AND $13) OR ($12 IS NULL AND NOT $13))
  AND (players.id = $14 OR $14 IS NULL)
  AND (records.submitter = $15 OR $15 IS NULL)
ORDER BY id {}
LIMIT $16