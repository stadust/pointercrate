-- This file should undo anything in `up.sql`

-- Fix up the views over the records table
DROP VIEW records_pds;
CREATE VIEW records_pds AS  -- records with player, demon and submitter
    SELECT records_pd.id, records_pd.progress, records_pd.video, records_pd.status_,
           records_pd.player_id, records_pd.player_name, records_pd.player_banned,
           records_pd.demon_id, records_pd.demon_name, records_pd.position,
           submitters.submitter_id, submitters.banned AS submitter_banned
    FROM records_pd
    INNER JOIN submitters
    ON records_pd.submitter_id = submitters.submitter_id;

ALTER TABLE records DROP COLUMN notes;