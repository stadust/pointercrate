UPDATE demons
SET level_id = NULL
WHERE level_id IS NOT NULL
    AND level_id NOT IN (
        SELECT level_id
        FROM gj_level
    );
-- if a demon was added when the constraint was dropped but had an invalid level_id (didn't appear in gj_level), it will be set to null
ALTER TABLE demons
ADD CONSTRAINT demons_level_id_fkey FOREIGN KEY (level_id) REFERENCES gj_level(level_id);