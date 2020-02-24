SELECT records.id, progress, records.video::text, status_::text AS status, records.notes,
       players.id AS player_id, players.name::text AS player_name, players.banned AS player_banned,
       demons.id AS demon_id, demons.name::text AS demon_name, demons.position,
       submitters.submitter_id AS submitter_id, submitters.banned AS submitter_banned
FROM records
INNER JOIN players ON records.player = players.id
INNER JOIN demons ON records.demon = demons.id
LEFT OUTER JOIN submitters ON records.submitter = submitters.submitter_id
WHERE records.id = $1