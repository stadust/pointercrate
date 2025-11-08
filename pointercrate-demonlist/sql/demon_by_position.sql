SELECT demons.id AS demon_id, demons.name AS "demon_name: String", demons.position, demons.rated_position, demons.requirement, demons.level_id, demons.rated, CASE WHEN verifiers.link_banned THEN NULL ElSE demons.video END, demons.thumbnail,
       verifiers.id AS verifier_id, verifiers.name AS "verifier_name: String", verifiers.banned AS verifier_banned,
       publishers.id AS publisher_id, publishers.name AS "publisher_name: String", publishers.banned AS publisher_banned
FROM demons
INNER JOIN players AS verifiers ON verifiers.id=demons.verifier
INNER JOIN players AS publishers ON publishers.id=demons.publisher
WHERE (demons.position=$1 AND $2) OR (demons.rated_position=$1 AND NOT $2)