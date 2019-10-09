-- Your SQL goes here

ALTER TABLE records ADD COLUMN notes TEXT;

-- Fix up the views over the records table
DROP VIEW records_pds;

CREATE VIEW records_pds AS  -- records with player, demon and submitter
    SELECT records.id, records.progress, records.video, records.status_, records.notes,
           players.id AS player_id, players.name AS player_name, players.banned AS player_banned,
           demons.id AS demon_id, demons.name AS demon_name, demons.position,
           submitters.submitter_id, submitters.banned AS submitter_banned
    FROM records
    INNER JOIN submitters
    ON records.submitter = submitters.submitter_id
    INNER JOIN players
    ON records.player = players.id
    INNER JOIN demons
    ON demons.id = records.demon;
