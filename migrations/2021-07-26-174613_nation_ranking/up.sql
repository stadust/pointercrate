-- Your SQL goes here

CREATE OR REPLACE FUNCTION best_records_in(country VARCHAR(2))
    RETURNS TABLE (LIKE records)
    AS
$body$
    SELECT DISTINCT ON (demon) records.*
    FROM records
    INNER JOIN players
    ON players.id = player
    WHERE status_='APPROVED' AND players.nationality = country
    ORDER BY demon, progress DESC;
$body$
LANGUAGE SQL;

CREATE OR REPLACE VIEW nations_with_score AS
    SELECT RANK() OVER(ORDER BY r.total_score DESC) AS rank,
           r.total_score AS score,
           nationalities.iso_country_code,
           nationalities.nation,
           nationalities.continent
    FROM (
             SELECT nationality,
                    SUM(scores.total_score) AS total_score
             FROM (
                      SELECT pseudo_records.player,
                             SUM(record_score(pseudo_records.progress::FLOAT, pseudo_records.position::FLOAT,
                                              100::FLOAT, pseudo_records.requirement)) as total_score
                      FROM (
                               SELECT player,
                                      progress,
                                      position,
                                      CASE WHEN demons.position > 75 THEN 100 ELSE requirement END AS requirement
                               FROM records
                                        INNER JOIN demons
                                                   ON demons.id = demon
                               WHERE demons.position <= 150
                                 AND status_ = 'APPROVED'

                               UNION

                               SELECT verifier                                                              as player,
                                      CASE WHEN demons.position > 150 THEN 0.0::FLOAT ELSE 100.0::FLOAT END as progress,
                                      position,
                                      100.0::FLOAT
                               FROM demons
                           ) AS pseudo_records
                      GROUP BY player
                  ) scores
                      INNER JOIN players
                                 ON scores.player = players.id
             WHERE NOT players.banned
               AND nationality IS NOT NULL
             GROUP BY nationality
         ) r
INNER JOIN nationalities
        ON nationalities.iso_country_code = r.nationality;
