SELECT ratedplus_index as index, ratedplus_rank as rank, id, name, ratedplus_score as score, subdivision, iso_country_code, nation
FROM ranked_players
WHERE ratedplus_rank IS NOT NULL
  AND (ratedplus_index < $1 OR $1 IS NULL)
  AND (ratedplus_index > $2 OR $2 IS NULL)
  AND (STRPOS(name, $3::CITEXT) > 0 OR $3 is NULL)
  AND (nation = $4 OR iso_country_code = $4 OR (nation IS NULL AND $5) OR ($4 IS NULL AND NOT $5))
  AND (continent = CAST($6::TEXT AS continent) OR $6 IS NULL)
  AND (subdivision = $7 OR $7 IS NULL)
ORDER BY ratedplus_rank {}, id
LIMIT $8