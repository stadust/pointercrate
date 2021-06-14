-- This file should undo anything in `up.sql`

-- Undo audit log related changes

CREATE OR REPLACE FUNCTION audit_player_modification() RETURNS trigger as $player_modification_trigger$
DECLARE
    name_change CITEXT;
    banned_change BOOLEAN;
BEGIN
    IF (OLD.name <> NEW.name) THEN
        name_change = OLD.name;
    END IF;

    IF (OLD.banned <> NEW.banned) THEN
        banned_change = OLD.banned;
    END IF;

    INSERT INTO player_modifications (userid, id, name, banned)
        (SELECT id, NEW.id, name_change, banned_change FROM active_user LIMIT 1);

    RETURN NEW;
END;
$player_modification_trigger$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION audit_player_deletion() RETURNS trigger AS $player_deletion_trigger$
BEGIN
    INSERT INTO player_modifications (userid, id, name, banned)
        (SELECT id, OLD.id, OLD.name, OLD.banned
         FROM active_user LIMIT 1);

    INSERT INTO player_deletions (userid, id)
        (SELECT id, OLD.id FROM active_user LIMIT 1);

    RETURN NULL;
END;
$player_deletion_trigger$ LANGUAGE plpgsql;

ALTER TABLE player_modifications DROP COLUMN nationality, DROP COLUMN subdivision;

-- cannot drop columns from VIEW via CREATE OR REPLACE

DROP VIEW players_with_score;
CREATE OR REPLACE VIEW players_with_score AS
SELECT players.id,
       players.name,
       RANK() OVER(ORDER BY scores.total_score DESC) AS rank,
       CASE WHEN scores.total_score IS NULL THEN 0.0::FLOAT ELSE scores.total_score END AS score,
       ROW_NUMBER() OVER(ORDER BY scores.total_score DESC) AS index,
       nationalities.iso_country_code,
       nationalities.nation
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
                 WHERE demons.position <= 150 AND status_ = 'APPROVED'

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

ALTER TABLE nationalities DROP COLUMN continent;

DROP TABLE subdivisions;
DROP TYPE continent;

ALTER TABLE players DROP COLUMN subdivision;