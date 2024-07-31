SELECT progress,
       CASE WHEN players.link_banned THEN NULL ELSE records.video::text END,
       CASE WHEN players.link_banned THEN NULL ELSE records.raw_footage::text END,
       status_::text AS "status!: String" ,
       players.id AS player_id, players.name AS "player_name: String", players.banned AS player_banned,
       demons.id AS demon_id, demons.name AS "demon_name: String", demons.position,
       submitters.submitter_id AS submitter_id, submitters.banned AS submitter_banned
FROM records
INNER JOIN players ON records.player = players.id
INNER JOIN demons ON records.demon = demons.id
INNER JOIN submitters ON records.submitter = submitters.submitter_id
WHERE records.id = $1
