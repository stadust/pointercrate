-- Your SQL goes here

-- This is a workaround
-- To get the new backend to work, we need to run all migrations against the existing database
-- all things from this migration already exist in that database, so theoretically, it should be a NOOP.
-- However, there is no `CREATE ... IF NOT EXISTS` equavalent for types in postgres
-- The following code tries to create the type and ignores the error by explicitly registering a handler for it
DO $$ BEGIN
    CREATE TYPE RECORD_STATUS AS ENUM ('APPROVED', 'REJECTED', 'SUBMITTED', 'DELETED');
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

CREATE EXTENSION CITEXT;

CREATE TABLE IF NOT EXISTS  players(
    id SERIAL PRIMARY KEY,
    name CITEXT NOT NULL UNIQUE,
    banned BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE TABLE IF NOT EXISTS  submitters (
    submitter_id SERIAL PRIMARY KEY,
    ip_address INET NOT NULL,
    banned BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE TABLE IF NOT EXISTS  members (
    member_id SERIAL PRIMARY KEY,
    name CITEXT UNIQUE NOT NULL,
    display_name CITEXT NULL DEFAULT NULL,
    youtube_channel VARCHAR(200) NULL DEFAULT NULL,
    password_hash BYTEA NOT NULL,
    password_salt BYTEA NOT NULL,
    permissions BIT(16) NOT NULL DEFAULT B'0000000000000000'::BIT(16)
);

CREATE TABLE IF NOT EXISTS  demons (
    name CITEXT PRIMARY KEY,
    position SMALLINT NOT NULL,
    requirement SMALLINT NOT NULL,
    video VARCHAR(200),
    description TEXT NULL,
    notes TEXT NULL,
    verifier INT NOT NULL REFERENCES players(id) ON DELETE RESTRICT ON UPDATE CASCADE,
    publisher INT NOT NULL REFERENCES players(id) ON DELETE RESTRICT ON UPDATE CASCADE,

    CONSTRAINT unique_position UNIQUE (position) DEFERRABLE INITIALLY IMMEDIATE,
    CONSTRAINT valid_record_req CHECK (requirement >= 0 AND requirement <= 100)
);

CREATE TABLE IF NOT EXISTS  records (
    id SERIAL PRIMARY KEY,
    progress SMALLINT CHECK (progress >= 0 AND progress <= 100) NOT NULL,
    video VARCHAR(200) UNIQUE,
    status_ RECORD_STATUS NOT NULL,
    player INT NOT NULL REFERENCES players(id) ON DELETE RESTRICT ON UPDATE CASCADE,
    submitter INT NOT NULL REFERENCES submitters(submitter_id) ON DELETE RESTRICT,
    demon CITEXT NOT NULL REFERENCES demons(name) ON DELETE CASCADE ON UPDATE CASCADE,
    CONSTRAINT records_demon_player_status__key UNIQUE (demon, player, status_) DEFERRABLE INITIALLY IMMEDIATE
);

CREATE TABLE IF NOT EXISTS  creators (
    demon CITEXT NOT NULL REFERENCES demons(name) ON DELETE RESTRICT ON UPDATE CASCADE,
    creator INT NOT NULL REFERENCES players(id) ON DELETE RESTRICT ON UPDATE CASCADE,
    PRIMARY KEY (demon, creator)
);

GRANT TRIGGER, REFERENCES, SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA public TO pointercrate;
GRANT USAGE ON ALL SEQUENCES IN SCHEMA public TO pointercrate;