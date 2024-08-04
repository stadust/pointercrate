-- Add up migration script here
DROP FUNCTION best_records_in(VARCHAR(2));

CREATE FUNCTION best_records_in(country VARCHAR(2))
    RETURNS TABLE (
        id integer ,
        progress smallint ,
        video character varying(200),
        status_ public.record_status ,
        player integer ,
        submitter integer ,
        demon integer
    )
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