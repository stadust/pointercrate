-- This file should undo anything in `up.sql`

-- Drop everything in reverse order

DROP TABLE audit_log;
DROP TABLE creators;
DROP TABLE records;
DROP TABLE demons;
DROP TABLE members;
DROP TABLE submitters;
DROP TABLE players;
DROP TYPE AUDIT_OPERATION;

DROP TYPE RECORD_STATUS;
DROP EXTENSION CITEXT;