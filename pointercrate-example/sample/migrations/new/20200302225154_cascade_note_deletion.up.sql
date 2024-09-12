-- Your SQL goes here

ALTER TABLE record_notes
    DROP CONSTRAINT record_notes_record_fkey,
    ADD CONSTRAINT record_notes_record_fkey
        FOREIGN KEY (record)
        REFERENCES records(id)
        ON DELETE CASCADE;