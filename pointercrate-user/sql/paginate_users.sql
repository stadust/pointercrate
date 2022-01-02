SELECT member_id, name, permissions::INTEGER, display_name::TEXT, youtube_channel::TEXT
FROM members
WHERE (member_id < $1 OR $1 IS NULL)
  AND (member_id > $2 OR $2 is NULL)
  AND (name = $3 OR $3 IS NULL)
  AND (display_name = $4 OR (display_name IS NULL AND $5) OR ($4 IS NULL AND NOT $5))
  AND (permissions & CAST($6::INTEGER AS BIT(16)) = CAST($6::INTEGER AS BIT(16)) OR $6 IS NULL)
  AND (permissions & CAST($7::INTEGER AS BIT(16)) <> 0::BIT(16) OR $7 IS NULL)
  AND (STRPOS(name, $8::CITEXT) > 0 OR $8 is NULL)
ORDER BY member_id {}
LIMIT $9
-- This entire query works because every comparison with NULL not done via IS evaluated to NULL, and NULL is false-y