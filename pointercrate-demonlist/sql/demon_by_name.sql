SELECT demons.id AS demon_id, demons.name AS "demon_name: String", demons.position, demons.requirement, demons.level_id, CASE WHEN verifiers.link_banned THEN NULL ElSE demons.video::text END,
       verifiers.id AS verifier_id, verifiers.name AS "verifier_name: String", verifiers.banned AS verifier_banned,
       publishers.id AS publisher_id, publishers.name AS "publisher_name: String", publishers.banned AS publisher_banned
FROM demons
INNER JOIN players AS verifiers ON verifiers.id=demons.verifier
INNER JOIN players AS publishers ON publishers.id=demons.publisher
WHERE demons.name=$1::CITEXT