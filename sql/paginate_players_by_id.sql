SELECT id, name::TEXT, banned, nation::TEXT, iso_country_code::TEXT
FROM players
LEFT OUTER JOIN nationalities ON nationality = iso_country_code
WHERE (id < $1 OR $1 IS NULL)
  AND (id > $2 OR $2 IS NULL)
  AND (name = $3::CITEXT OR $3 is NULL)
  AND (STRPOS(name, $4::CITEXT) > 0 OR $4 is NULL)
  AND (banned = $5 OR $5 IS NULL)
  AND (nationality = $6 OR iso_country_code = $6 OR (nationality IS NULL AND $7) OR ($6 IS NULL AND NOT $7))
ORDER BY id {}
LIMIT $8