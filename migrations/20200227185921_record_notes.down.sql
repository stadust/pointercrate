-- This file should undo anything in `up.sql`

ALTER TABLE records ADD COLUMN notes TEXT;

WITH transferred_notes AS (
    SELECT record, STRING_AGG(content, '\n') AS content
    FROM record_notes
    GROUP BY record
)
UPDATE records
SET notes = transferred_notes.content
FROM transferred_notes
WHERE records.id = transferred_notes.record;

DROP TABLE record_notes;

DROP TABLE record_notes_additions;
DROP FUNCTION audit_record_notes_addition() CASCADE;

DROP TABLE record_notes_modifications;
DROP FUNCTION audit_record_notes_modification() CASCADE;

DROP TABLE record_notes_deletions;
DROP FUNCTION audit_record_notes_deletion() CASCADE;