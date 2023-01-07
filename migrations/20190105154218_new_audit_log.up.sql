-- Alright, audit_log overhault
-- In audit log entries we cannot use foreign keys because that would force us to delete log entries
-- when objects are removed from the db, or disallow the deletion of objects
-- Thus, we need to make sure we references database objects by a non-modifiable primary key, which
-- we do not actually use as foreign key
--
-- The problem here is, that the "demons" relation doesn't have such a column -
-- position and name (while both unique and even primary in "name"'s case') are modifiable
-- This is the reason we do not allow the deletion of demons as of right now
--
-- (Once the new backend is running and we no longer need to be backwards compatible with the
-- python codebase, we can ofc modify the demons table)

-- Global fallback table in case no temporary 'active_user' table was created in the current session
CREATE TABLE active_user (id INTEGER PRIMARY KEY); -- primary key required because diesel migrations break otherwise

-- TODO: generate a dummy member with ID 0
INSERT INTO active_user VALUES (0);

CREATE TABLE audit_log2 (
    time TIMESTAMP WITHOUT TIME ZONE DEFAULT (NOW() AT TIME ZONE 'utc') NOT NULL,
    audit_id SERIAL PRIMARY KEY NOT NULL,
    userid INTEGER NOT NULL -- REFERENCES members(member_id)
);

CREATE TABLE demon_additions (
    -- Note that this currently forces us to not delete demons
    name CITEXT REFERENCES demons(name) ON DELETE RESTRICT ON UPDATE CASCADE NOT NULL
) INHERITS (audit_log2);

CREATE FUNCTION audit_demon_addition() RETURNS trigger AS $demon_add_trigger$
    BEGIN
        INSERT INTO demon_additions (userid, name) (SELECT id, NEW.name FROM active_user LIMIT 1);
        RETURN NEW;
    END;
$demon_add_trigger$ LANGUAGE plpgsql;

CREATE TRIGGER demon_addition_trigger AFTER INSERT ON demons FOR EACH ROW EXECUTE PROCEDURE audit_demon_addition();

CREATE TABLE demon_modifications (
    -- Column that keeps track of the demon that was modified
    -- Note that this currently forces us to not delete demons
    demon CITEXT REFERENCES demons(name) ON DELETE RESTRICT ON UPDATE CASCADE NOT NULL,

    -- Column that keeps track of changes to the demon's name
    name CITEXT NULL,
    position SMALLINT NULL,
    requirement SMALLINT NULL,
    video VARCHAR(200) NULL,
    verifier INT NULL,
    publisher INT NULL
) INHERITS (audit_log2);

CREATE FUNCTION audit_demon_modification() RETURNS trigger AS $demon_modification_trigger$
    DECLARE
        name_change CITEXT;
        position_change SMALLINT;
        requirement_change SMALLINT;
        video_change VARCHAR(200);
        verifier_change SMALLINT;
        publisher_change SMALLINT;
    BEGIN
        IF (OLD.name <> NEW.name) THEN
            name_change = OLD.name;
        END IF;

        IF (OLD.position <> NEW.position) THEN
            position_change = OLD.position;
        END IF;

        IF (OLD.requirement <> NEW.requirement) THEN
            requirement_change = OLD.requirement;
        END IF;

        IF (OLD.video <> NEW.video) THEN
            video_change = OLD.video;
        END IF;

        IF (OLD.verifier <> NEW.verifier) THEN
            verifier_change = OLD.verifier;
        END IF;

        IF (OLD.publisher <> NEW.publisher) THEN
            publisher_change = OLD.publisher;
        END IF;

        INSERT INTO demon_modifications (userid, demon, name, position, requirement, video, verifier, publisher)
            (SELECT id, NEW.name, name_change, position_change, requirement_change, video_change, verifier_change, publisher_change
            FROM active_user LIMIT 1);

        RETURN NEW;
    END;
$demon_modification_trigger$ LANGUAGE plpgsql;

CREATE TRIGGER demon_modification_trigger AFTER UPDATE ON demons FOR EACH ROW EXECUTE PROCEDURE audit_demon_modification();

CREATE TABLE record_additions (
    id INTEGER NOT NULL -- REFERENCES records(id)
) INHERITS (audit_log2);

CREATE FUNCTION audit_record_addition() RETURNS trigger AS $record_add_trigger$
    BEGIN
        INSERT INTO record_additions (userid, id) (SELECT id, NEW.id FROM active_user LIMIT 1);
        RETURN NEW;
    END;
$record_add_trigger$ LANGUAGE plpgsql;

CREATE TRIGGER record_addition_trigger AFTER INSERT ON records FOR EACH ROW EXECUTE PROCEDURE audit_record_addition();

CREATE TABLE record_modifications (
    id INTEGER NOT NULL, -- REFERENCES records(id)

    progress SMALLINT NULL,
    video VARCHAR(200) NULL,
    status_ RECORD_STATUS NULL,
    player INT NULL, -- REFERENCES players(id)
    demon CITEXT NULL -- REFERENCES demons(name)
) INHERITS (audit_log2);

CREATE FUNCTION audit_record_modification() RETURNS trigger AS $record_modification_trigger$
    DECLARE
        progress_change SMALLINT;
        video_change VARCHAR(200);
        status_change RECORD_STATUS;
        player_change INT;
        demon_change CITEXT;
    BEGIN
        if (OLD.progress <> NEW.progress) THEN
            progress_change = OLD.progress;
        END IF;

        IF (OLD.video <> NEW.video) THEN
            video_change = OLD.video;
        END IF;

        IF (OLD.status_ <> NEW.status_) THEN
            status_change = OLD.status_;
        END IF;

        IF (OLD.player <> NEW.player) THEN
            player_change = OLD.player;
        END IF;

        IF (OLD.demon <> NEW.demon) THEN
            demon_change = OLD.demon;
        END IF;

        INSERT INTO record_modifications (userid, id, progress, video, status_, player, demon)
            (SELECT id, NEW.id, progress_change, video_change, status_change, player_change, demon_change
            FROM active_user LIMIT 1);

        RETURN NEW;
    END;
$record_modification_trigger$ LANGUAGE plpgsql;

CREATE TRIGGER record_modification_trigger AFTER UPDATE ON records FOR EACH ROW EXECUTE PROCEDURE audit_record_modification();

-- Before deletion we add a `record_modifications` entry that's a copy of the record directly before deletion
CREATE TABLE record_deletions (
    id INTEGER NOT NULL -- REFERENCES records(id)
) INHERITS (audit_log2);

CREATE FUNCTION audit_record_deletion() RETURNS trigger AS $record_deletion_trigger$
    BEGIN
        INSERT INTO record_modifications (userid, id, progress, video, status_, player, demon)
            (SELECT id, OLD.id, OLD.progress, OLD.video, OLD.status_, OLD.player, OLD.demon
            FROM active_user LIMIT 1);

        INSERT INTO record_deletions (userid, id)
            (SELECT id, OLD.id FROM active_user LIMIT 1);

        RETURN NULL;
    END;
$record_deletion_trigger$ LANGUAGE plpgsql;

CREATE TRIGGER record_deletion_trigger AFTER DELETE ON records FOR EACH ROW EXECUTE PROCEDURE audit_record_deletion();

CREATE TABLE player_additions (
    id INTEGER NOT NULL -- REFERENCES players(id)
) INHERITS (audit_log2);

CREATE FUNCTION audit_player_addition() RETURNS trigger AS $record_addition_trigger$
    BEGIN
        INSERT INTO player_additions(userid, id)
        (SELECT id, NEW.id FROM active_user LIMIT 1);

        RETURN NEW;
    END;
$record_addition_trigger$ LANGUAGE plpgsql;

CREATE TRIGGER player_addition_trigger AFTER INSERT ON players FOR EACH ROW EXECUTE PROCEDURE audit_player_addition();

CREATE TABLE player_modifications (
    id INTEGER NOT NULL, -- REFERENCES players(id)

    name CITEXT NULL,
    banned BOOLEAN NULL
) INHERITS (audit_log2);

CREATE FUNCTION audit_player_modification() RETURNS trigger as $player_modification_trigger$
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

CREATE TRIGGER player_modification_trigger AFTER UPDATE ON players FOR EACH ROW EXECUTE PROCEDURE audit_player_modification();

-- See handling of record_deletions
CREATE TABLE player_deletions (
    id INTEGER NOT NULL -- REFERENCES players(id)
) INHERITS (audit_log2);

CREATE FUNCTION audit_player_deletion() RETURNS trigger AS $player_deletion_trigger$
    BEGIN
        INSERT INTO player_modifications (userid, id, name, banned)
            (SELECT id, OLD.id, OLD.name, OLD.banned
            FROM active_user LIMIT 1);

        INSERT INTO player_deletions (userid, id)
            (SELECT id, OLD.id FROM active_user LIMIT 1);

        RETURN NULL;
    END;
$player_deletion_trigger$ LANGUAGE plpgsql;

CREATE TRIGGER player_deletion_trigger AFTER DELETE ON players FOR EACH ROW EXECUTE PROCEDURE audit_player_deletion();

CREATE TABLE creator_additions (
    creator INTEGER NOT NULL,
    demon CITEXT NOT NULL
) INHERITS (audit_log2);

CREATE FUNCTION audit_creator_addition() RETURNS trigger AS $audit_creator_addition$
    BEGIN
        INSERT INTO creator_additions (userid, creator, demon)
            (SELECT id, NEW.creator, NEW.demon
            FROM active_user LIMIT 1);

        RETURN NEW;
    END;
$audit_creator_addition$ LANGUAGE plpgsql;

CREATE TRIGGER creator_addition_trigger AFTER INSERT ON creators FOR EACH ROW EXECUTE PROCEDURE audit_creator_addition();

CREATE TABLE creator_deletions(
    creator INTEGER NOT NULL,
    demon CITEXT NOT NULL
) INHERITS (audit_log2);

CREATE FUNCTION audit_creator_deletion() RETURNS trigger AS $creator_deletion_trigger$
    BEGIN
        INSERT INTO creator_deletions (userid, creator, demon)
            (SELECT id, OLD.creator, OLD.demon
            FROM active_user LIMIT 1);

        RETURN NULL;
    END;
$creator_deletion_trigger$ LANGUAGE plpgsql;

CREATE TRIGGER creator_deletion_trigger AFTER DELETE ON creators FOR EACH ROW EXECUTE PROCEDURE audit_creator_deletion();

CREATE TABLE submitter_modifications (
    submitter INTEGER NOT NULL,

    banned BOOLEAN NULL
) INHERITS (audit_log2);

CREATE FUNCTION audit_submitter_modification() RETURNS trigger as $submitter_modifications_trigger$
    DECLARE
        banned_change BOOLEAN;
    BEGIN
        IF (OLD.banned <> NEW.banned) THEN
            banned_change = OLD.banned;
        END IF;

        INSERT INTO submitter_modifications (userid, submitter, banned)
        (SELECT id, NEW.submitter_id, banned_change FROM active_user LIMIT 1);

        RETURN NEW;
    END;
$submitter_modifications_trigger$ LANGUAGE plpgsql;

CREATE TRIGGER submitter_modification_trigger AFTER UPDATE ON submitters FOR EACH ROW EXECUTE PROCEDURE audit_submitter_modification();

CREATE TABLE user_additions (
    id INTEGER NOT NULL -- REFERENCES members(member_id)
) INHERITS (audit_log2);

CREATE FUNCTION audit_user_addition() RETURNS trigger AS $audit_user_addition$
    BEGIN
        -- cannot be logged in during registration
        INSERT INTO user_additions (userid, id) VALUES (0, NEW.member_id);

        RETURN NEW;
    END;
$audit_user_addition$ LANGUAGE plpgsql;

CREATE TRIGGER user_addition_trigger AFTER INSERT ON members FOR EACH ROW EXECUTE PROCEDURE audit_user_addition();

CREATE TABLE user_modifications (
    id INTEGER NOT NULL, -- REFERENCES members(member_id)

    -- fields updatable by user themself
    display_name CITEXT NULL,
    youtube_channel CITEXT NULL,

    -- fields updatable by staff
    permissions BIT(16) NULL
) INHERITS (audit_log2);

CREATE FUNCTION audit_user_modification() RETURNS trigger as $user_modification_trigger$
    DECLARE
        display_name_change CITEXT;
        youtube_channel_change BOOLEAN;
        permissions_change BIT(16);
    BEGIN
        IF (OLD.display_name <> NEW.display_name) THEN
            display_name_change = OLD.display_name;
        END IF;

        IF (OLD.youtube_channel <> NEW.youtube_channel) THEN
            youtube_channel_change = OLD.youtube_channel;
        END IF;

        IF (OLD.permissions <> NEW.permissions) THEN
            permissions_change = OLD.permissions;
        END IF;

        INSERT INTO user_modifications (userid, id, display_name, youtube_channel, permissions)
        (SELECT id, NEW.member_id, display_name_change, youtube_channel_change, permissions_change FROM active_user LIMIT 1);

        RETURN NEW;
    END;
$user_modification_trigger$ LANGUAGE plpgsql;

CREATE TRIGGER user_modification_trigger AFTER UPDATE ON members FOR EACH ROW EXECUTE PROCEDURE audit_user_modification();

CREATE TABLE user_deletions (
    id INTEGER NOT NULL -- REFERENCES members(member_id)
) INHERITS (audit_log2);

CREATE FUNCTION audit_user_deletion() RETURNS trigger AS $user_deletion_trigger$
    BEGIN
        INSERT INTO user_modifications (userid, id, display_name, youtube_channel, permissions)
            (SELECT id, OLD.member_id, OLD.display_name, OLD.youtube_channel, OLD.permissions
            FROM active_user LIMIT 1);

        INSERT INTO user_deletions (userid, id)
            (SELECT id, OLD.member_id FROM active_user LIMIT 1);

        RETURN NULL;
    END;
$user_deletion_trigger$ LANGUAGE plpgsql;

CREATE TRIGGER user_deletion_trigger AFTER DELETE ON members FOR EACH ROW EXECUTE PROCEDURE audit_user_deletion();
