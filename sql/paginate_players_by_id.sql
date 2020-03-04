SELECT id, name::TEXT, banned, nation::TEXT, iso_country_code::TEXT
FROM players
INNER JOIN nationalities ON nationality = iso_country_code
WHERE (id < $1 OR $1 IS NULL)
  AND (id > $2 OR $2 IS NULL)
  AND (name = $3::CITEXT OR $3 is NULL)
  AND (banned = $4 OR $4 IS NULL)
  AND (nation = $5 OR iso_country_code = $5 OR (nationality IS NULL AND $6) OR ($5 IS NULL AND NOT $6))
ORDER BY id {}
LIMIT $7