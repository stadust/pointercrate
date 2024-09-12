-- Your SQL goes here

CREATE OR REPLACE FUNCTION best_records_in(country VARCHAR(2))
    RETURNS TABLE (LIKE records)
    AS
$body$
    WITH grp AS (
        SELECT records.*,
               RANK() OVER (PARTITION BY demon ORDER BY demon, progress DESC) AS rk
        FROM records
        INNER JOIN players
        ON players.id = player
        WHERE status_='APPROVED' AND players.nationality = country
    )
    SELECT id, progress, video, status_, player, submitter, demon
    FROM grp
    WHERE rk = 1;
$body$
LANGUAGE SQL;

CREATE OR REPLACE VIEW nations_with_score AS
    SELECT RANK() OVER(ORDER BY scores.total_score DESC) AS rank,
           scores.total_score AS score,
           nationalities.iso_country_code,
           nationalities.nation,
           nationalities.continent
    FROM (
          SELECT nationality,
                 SUM(record_score(pseudo_records.progress::FLOAT, pseudo_records.position::FLOAT,
                                  100::FLOAT, pseudo_records.requirement)) as total_score
          FROM (
                   select distinct on (nationality, demon)
                       nationality,
                       progress,
                       position,
                       CASE WHEN demons.position > 75 THEN 100 ELSE requirement END AS requirement
                   from (
                       select demon, player, progress
                       from records
                       where status_='APPROVED'

                       union

                       select id, verifier, 100
                       from demons
                   ) records
                       inner join demons
                           on demons.id = records.demon
                       inner join players
                           on players.id=records.player
                       inner join nationalities
                           on iso_country_code=players.nationality
                   where position <= 150 and not players.banned
                   order by nationality, demon, progress desc
               ) AS pseudo_records
          GROUP BY nationality
   ) scores
INNER JOIN nationalities
        ON nationalities.iso_country_code = scores.nationality;
