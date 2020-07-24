SELECT demons.id AS demon_id, demons.name::text AS demon_name, demons.position, demons.requirement, CASE WHEN verifiers.link_banned THEN NULL ElSE demons.video::text END,
       verifiers.id AS verifier_id, verifiers.name::text AS verifier_name, verifiers.banned AS verifier_banned,
       publishers.id AS publisher_id, publishers.name::text AS publisher_name, publishers.banned AS publisher_banned
FROM demons
INNER JOIN players AS verifiers ON verifiers.id=demons.verifier
INNER JOIN players AS publishers ON publishers.id=demons.publisher
WHERE (demons.position < $1 OR $1 IS NULL)
  AND (demons.position > $2 OR $2 IS NULL)
  AND (demons.name::CITEXT = $3 OR $3 IS NULL)
  AND (requirement = $4 OR $4 IS NULL)
  AND (requirement < $5 OR $5 IS NULL)
  AND (requirement > $6 OR $6 IS NULL)
  AND (verifiers.id = $7 OR $7 IS NULL)
  AND (verifiers.name::CITEXT = $8 OR $8 IS NULL)
  AND (publishers.id = $9 OR $9 IS NULL)
  AND (publishers.name::CITEXT = $10 OR $10 IS NULL)
  AND (STRPOS(demons.name, $11::CITEXT) > 0 OR $11 is NULL)
  AND demons.position IS NOT NULL
ORDER BY demons.position {}
LIMIT $12