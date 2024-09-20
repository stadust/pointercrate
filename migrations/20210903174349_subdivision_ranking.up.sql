-- Your SQL goes here

CREATE OR REPLACE FUNCTION best_records_local(country VARCHAR(2), the_subdivision VARCHAR(3))
    RETURNS TABLE (LIKE records)
AS
$body$
WITH grp AS (
    SELECT records.*,
           RANK() OVER (PARTITION BY demon ORDER BY demon, progress DESC) AS rk
    FROM records
        INNER JOIN players
            ON players.id = player
    WHERE status_='APPROVED' AND players.nationality = country AND players.subdivision = the_subdivision
)
SELECT id, progress, video, status_, player, submitter, demon
FROM grp
WHERE rk = 1;
$body$
    LANGUAGE SQL;

CREATE OR REPLACE FUNCTION subdivision_ranking_of(country VARCHAR(2))
    RETURNS TABLE (
        rank BIGINT,
        score FLOAT,
        subdivision_code VARCHAR(3),
        name TEXT
    )
AS
    $body$
    SELECT RANK() OVER(ORDER BY scores.total_score DESC) AS rank,
           scores.total_score AS score,
           iso_code,
           name
    FROM (
        SELECT iso_code, name,
                SUM(record_score(pseudo_records.progress::FLOAT, pseudo_records.position::FLOAT,
                                 100::FLOAT, pseudo_records.requirement)) as total_score
         FROM (
                  select distinct on (iso_code, demon)
                      iso_code,
                      subdivisions.name,
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
                           inner join subdivisions
                                      on (iso_code=players.subdivision and players.nationality = nation)
                  where position <= 150 and not players.banned and nation = country
                  order by iso_code, demon, progress desc
              ) AS pseudo_records
         GROUP BY iso_code, name
     ) scores;
    $body$
LANGUAGE SQL;

