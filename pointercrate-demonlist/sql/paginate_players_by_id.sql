SELECT id, players.name::TEXT, banned, nationalities.nation::TEXT, iso_country_code::TEXT, subdivision::TEXT AS iso_code, subdivisions.name AS subdivision_name, players.score
FROM players
LEFT OUTER JOIN nationalities ON nationality = iso_country_code
LEFT JOIN subdivisions ON iso_code = subdivision AND subdivisions.nation = nationality
WHERE (id < $1 OR $1 IS NULL)
  AND (id > $2 OR $2 IS NULL)
  AND (players.name = $3::CITEXT OR $3 is NULL)
  AND (STRPOS(players.name, $4::CITEXT) > 0 OR $4 is NULL)
  AND (banned = $5 OR $5 IS NULL)
  AND (nationality = $6 OR iso_country_code = $6 OR (nationality IS NULL AND $7) OR ($6 IS NULL AND NOT $7))
  AND (subdivision = $8 OR $8 IS NULL)
ORDER BY id {}
LIMIT $9