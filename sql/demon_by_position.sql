SELECT demons.id AS demon_id, demons.name::text AS demon_name, demons.position, demons.requirement, CASE WHEN verifiers.link_banned THEN NULL ElSE demons.video::text END,
       verifiers.id AS verifier_id, verifiers.name::text AS verifier_name, verifiers.banned AS verifier_banned,
       publishers.id AS publisher_id, publishers.name::text AS publisher_name, publishers.banned AS publisher_banned
FROM demons
INNER JOIN players AS verifiers ON verifiers.id=demons.verifier
INNER JOIN players AS publishers ON publishers.id=demons.publisher
WHERE demons.position=$1