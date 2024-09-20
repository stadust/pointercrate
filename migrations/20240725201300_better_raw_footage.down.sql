ALTER TABLE records DROP COLUMN raw_footage;

DROP FUNCTION best_records_in(VARCHAR(2));

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
