DROP FUNCTION list_at(TIMESTAMP WITHOUT TIME ZONE);

CREATE FUNCTION list_at(is_rated_list BOOLEAN, TIMESTAMP WITHOUT TIME ZONE)
    RETURNS TABLE (
                      name CITEXT,
                      position_ SMALLINT,
                      requirement SMALLINT,
                      video VARCHAR(200),
                      thumbnail TEXT,
                      verifier INTEGER,
                      publisher INTEGER,
                      id INTEGER,
                      level_id BIGINT,
                      rated BOOLEAN,
                      current_position SMALLINT
                  )
AS $$
SELECT name, COALESCE(t.position, CASE WHEN is_rated_list THEN demons.rated_position ELSE demons.position END) as position_, requirement, video, thumbnail, verifier, publisher, demons.id, level_id, COALESCE(r.rated, demons.rated),
CASE WHEN is_rated_list THEN demons.rated_position ELSE demons.position END AS current_position
FROM demons
LEFT OUTER JOIN (
    SELECT DISTINCT ON (id) id, CASE WHEN is_rated_list THEN rated_position ELSE position END AS position
    FROM demon_modifications
    WHERE time >= $2 AND (
        (is_rated_list AND rated_position IS NOT NULL) 
        OR (NOT is_rated_list AND position != -1)
    )
    ORDER BY id, time
) t
ON demons.id = t.id
LEFT OUTER JOIN (
    SELECT DISTINCT ON (id) id, rated
    FROM demon_modifications
    WHERE time >= $2 AND rated IS NOT NULL
    ORDER BY id, time
) r
ON demons.id = r.id
WHERE NOT EXISTS (SELECT 1 FROM demon_additions WHERE demon_additions.id = demons.id AND time >= $2)
AND (NOT is_rated_list OR COALESCE(r.rated, demons.rated))
$$
    LANGUAGE SQL
    STABLE;
