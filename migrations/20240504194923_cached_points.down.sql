-- Add down migration script here
DROP VIEW ranked_players;
DROP VIEW ranked_nations;

ALTER TABLE players DROP COLUMN score;
ALTER TABLE nationalities DROP COLUMN score;
ALTER TABLE subdivisions DROP COLUMN score;

DROP FUNCTION recompute_player_scores();
DROP FUNCTION score_of_player(player_id INTEGER);
DROP FUNCTION recompute_nation_scores();
DROP FUNCTION score_of_nation(iso_country_code VARCHAR(2));
DROP FUNCTION recompute_subdivision_scores();
DROP FUNCTION score_of_subdivision(iso_country_code VARCHAR(2), iso_code VARCHAR(3));
DROP VIEW score_giving;

ALTER TABLE players DROP CONSTRAINT nation_subdivions_fkey;

-- Copied from 20210419002933.up
CREATE VIEW players_with_score AS
SELECT players.id,
       players.name,
       RANK() OVER(ORDER BY scores.total_score DESC) AS rank,
       CASE WHEN scores.total_score IS NULL THEN 0.0::FLOAT ELSE scores.total_score END AS score,
       ROW_NUMBER() OVER(ORDER BY scores.total_score DESC) AS index,
       nationalities.iso_country_code,
       nationalities.nation,
       players.subdivision,
       nationalities.continent
FROM
    (
        SELECT pseudo_records.player,
               SUM(record_score(pseudo_records.progress::FLOAT, pseudo_records.position::FLOAT, 100::FLOAT, pseudo_records.requirement)) as total_score
        FROM (
                 SELECT player,
                        progress,
                        position,
                        CASE WHEN demons.position > 75 THEN 100 ELSE requirement END AS requirement
                 FROM records
                          INNER JOIN demons
                                     ON demons.id = demon
                 WHERE demons.position <= 150 AND status_ = 'APPROVED' AND (demons.position <= 75 OR progress = 100)

                 UNION

                 SELECT verifier as player,
                        CASE WHEN demons.position > 150 THEN 0.0::FLOAT ELSE 100.0::FLOAT END as progress,
                        position,
                        100.0::FLOAT
                 FROM demons

                 UNION

                 SELECT publisher as player,
                        0.0::FLOAT as progress,
                        position,
                        100.0::FLOAT
                 FROM demons

                 UNION

                 SELECT creator as player,
                        0.0::FLOAT as progress,
                        1.0::FLOAT as position, -- doesn't matter
                        100.0::FLOAT
                 FROM creators
             ) AS pseudo_records
        GROUP BY player
    ) scores
        INNER JOIN players
                   ON scores.player = players.id
        LEFT OUTER JOIN nationalities
                        ON players.nationality = nationalities.iso_country_code
WHERE NOT players.banned AND players.id != 1534;

-- Copied from 20210726174613
CREATE VIEW nations_with_score AS
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

-- Copied from 20210903174349
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

