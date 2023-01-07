-- Your SQL goes here
CREATE TABLE record_notes (
    id SERIAL PRIMARY KEY,
    record INTEGER REFERENCES records(id) NOT NULL,
    content TEXT NOT NULL
);

INSERT INTO record_notes (record, content)
SELECT id, notes FROM records WHERE notes IS NOT NULL;

ALTER TABLE records DROP COLUMN notes;

-- This part of the migration cannot be undone!
ALTER TYPE record_status ADD VALUE 'UNDER_CONSIDERATION';

CREATE TABLE record_notes_additions (
    id INTEGER NOT NULL
) INHERITS (audit_log2);

CREATE TABLE record_notes_modifications (
    id INTEGER NOT NULL, -- which note was changed?
    record INTEGER NULL,
    content TEXT NULL
) INHERITS (audit_log2);

CREATE TABLE record_notes_deletions (
    id INTEGER NOT NULL
) INHERITS (audit_log2);

CREATE FUNCTION audit_record_notes_addition() RETURNS trigger AS $record_notes_add_trigger$
    BEGIN
        INSERT INTO record_notes_additions (userid, id) (SELECT id, NEW.id FROM active_user LIMIT 1);
        RETURN NEW;
    END;
$record_notes_add_trigger$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION audit_record_notes_modification() RETURNS trigger AS $record_notes_modification_trigger$
    DECLARE
        record_change INTEGER;
        content_change TEXT;
    BEGIN
        IF (OLD.record <> NEW.record) THEN
            record_change = OLD.record;
        END IF;

        IF (OLD.content <> NEW.content) THEN
            content_change = OLD.content;
        END IF;

        INSERT INTO record_notes_modifications (userid, id, record, content)
            (SELECT id, OLD.id, record_change, content_change FROM active_user LIMIT 1);

        RETURN NEW;
    END;
$record_notes_modification_trigger$ LANGUAGE plpgsql;

CREATE FUNCTION audit_record_notes_deletion() RETURNS trigger AS $record_notes_deletion_trigger$
    BEGIN
        INSERT INTO record_notes_modifications (userid, id, record, content)
            (SELECT id, OLD.id, OLD.record, OLD.content FROM active_user LIMIT 1);

        INSERT INTO record_notes_deletion (userid, id)
            (SELECT id, OLD.id FROM active_user LIMIT 1);

        RETURN NEW;
    END
$record_notes_deletion_trigger$ LANGUAGE plpgsql;

CREATE TRIGGER record_note_addition_trigger AFTER INSERT ON record_notes FOR EACH ROW EXECUTE PROCEDURE audit_record_notes_addition();
CREATE TRIGGER record_note_modification_trigger AFTER UPDATE ON record_notes FOR EACH ROW EXECUTE PROCEDURE audit_record_notes_modification();
CREATE TRIGGER record_note_deletion_trigger AFTER DELETE ON record_notes FOR EACH ROW EXECUTE PROCEDURE audit_record_notes_modification();