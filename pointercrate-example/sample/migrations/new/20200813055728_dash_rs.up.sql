-- Your SQL goes here

CREATE TABLE gj_creator (
    user_id bigint PRIMARY KEY NOT NULL,
    name text NOT NULL,
    account_id bigint
);

CREATE TABLE gj_creator_meta (
    user_id bigint PRIMARY KEY NOT NULL,  -- No REFERENCES creator(user_id) as we also have to keep track of _missing_ entries here!
    cached_at timestamp without time zone NOT NULL,
    absent boolean DEFAULT false NOT NULL
);

CREATE TABLE gj_level (
    level_id bigint PRIMARY KEY NOT NULL,
    level_name text NOT NULL,
    description text,
    level_version integer NOT NULL,
    creator_id bigint NOT NULL,
    difficulty smallint NOT NULL,
    is_demon boolean not null,
    downloads integer NOT NULL,
    main_song smallint,
    gd_version smallint NOT NULL,
    likes integer NOT NULL,
    level_length smallint NOT NULL,
    stars smallint NOT NULL,
    featured integer NOT NULL,
    copy_of bigint,
    two_player boolean NOT NULL,
    custom_song_id bigint,
    coin_amount smallint NOT NULL,
    coins_verified boolean NOT NULL,
    stars_requested smallint,
    is_epic boolean NOT NULL,
    object_amount integer,
    index_46 text,
    index_47 text
);

CREATE TABLE gj_level_meta (
    level_id bigint PRIMARY KEY NOT NULL,
    cached_at timestamp without time zone NOT NULL,
    absent boolean DEFAULT false NOT NULL
);

-- Many-to-many table mapping each level request to the list of levels it returned
CREATE TABLE gj_level_request_results (
    level_id bigint NOT NULL,
    request_hash bigint NOT NULL
);

CREATE TABLE gj_level_request_meta (
    request_hash bigint PRIMARY KEY NOT NULL,
    cached_at timestamp without time zone NOT NULL,
    absent boolean DEFAULT false NOT NULL
);

CREATE TABLE gj_level_data (
    level_id bigint PRIMARY KEY REFERENCES gj_level(level_id) NOT NULL,
    level_data bytea NOT NULL,
    level_password integer,
    time_since_upload text NOT NULL,
    time_since_update text NOT NULL,
    index_36 text
);

CREATE TABLE gj_level_data_meta (
  level_id bigint PRIMARY KEY NOT NULL,
  cached_at timestamp without time zone NOT NULL,
  absent boolean DEFAULT false NOT NULL
);

CREATE TABLE gj_newgrounds_song (
    song_id bigint PRIMARY KEY NOT NULL,
    song_name text NOT NULL,
    index_3 bigint NOT NULL,
    song_artist text NOT NULL,
    filesize double precision NOT NULL,
    index_6 text,
    index_7 text,
    index_8 text NOT NULL,
    song_link text NOT NULL
);

CREATE TABLE gj_newgrounds_song_meta (
    song_id bigint PRIMARY KEY NOT NULL,
    cached_at timestamp without time zone NOT NULL,
    absent boolean DEFAULT false NOT NULL
);

CREATE TABLE download_lock(
    level_id bigint not null
);

ALTER TABLE demons ADD COLUMN level_id INT8 NULL UNIQUE REFERENCES gj_level(level_id);