SELECT id, name::TEXT, rank, score, index, nation::TEXT, iso_country_code::TEXT
FROM players_with_score
WHERE (index < $1 OR $1 IS NULL)
  AND (index > $2 OR $2 IS NULL)
  AND (STRPOS(name, $3::CITEXT) > 0 OR $3 is NULL)
  AND (nation = $4 OR iso_country_code = $4 OR (nation IS NULL AND $5) OR ($4 IS NULL AND NOT $5))
ORDER BY rank {}
LIMIT $6