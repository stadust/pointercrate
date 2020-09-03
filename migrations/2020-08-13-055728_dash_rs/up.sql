-- Your SQL goes here

CREATE TABLE creator (
    user_id bigint PRIMARY KEY NOT NULL,
    name text NOT NULL,
    account_id bigint
);

CREATE TABLE creator_meta (
    user_id bigint PRIMARY KEY NOT NULL,  -- No REFERENCES creator(user_id) as we also have to keep track of _missing_ entries here!
    cached_at timestamp without time zone,
    absent boolean DEFAULT false NOT NULL
);

CREATE TABLE level (
    level_id bigint PRIMARY KEY NOT NULL,
    level_name text NOT NULL,
    description text,
    level_version integer NOT NULL,
    creator_id bigint NOT NULL,
    difficulty text NOT NULL,
    downloads integer NOT NULL,
    main_song smallint,
    gd_version smallint NOT NULL,
    likes integer NOT NULL,
    level_length text NOT NULL,
    stars smallint NOT NULL,
    featured integer NOT NULL,
    copy_of bigint,
    index_31 text,
    custom_song_id bigint,
    coin_amount smallint NOT NULL,
    coins_verified boolean NOT NULL,
    stars_requested smallint,
    is_epic boolean NOT NULL,
    index_43 text NOT NULL,
    object_amount integer,
    index_46 text,
    index_47 text
);

CREATE TABLE level_meta (
    level_id bigint PRIMARY KEY NOT NULL,
    cached_at timestamp without time zone,
    absent boolean DEFAULT false NOT NULL
);

-- Many-to-many table mapping each level request to the list of levels it returned
CREATE TABLE level_request_results (
    level_id bigint NOT NULL,
    request_hash bigint NOT NULL
);

CREATE TABLE level_request_meta (
    request_hash bigint PRIMARY KEY NOT NULL,
    cached_at timestamp without time zone,
    absent boolean DEFAULT false NOT NULL
);

CREATE TABLE level_data (
    level_id bigint PRIMARY KEY REFERENCES level(level_id) NOT NULL,
    level_data bytea NOT NULL,
    level_password text,
    time_since_upload text NOT NULL,
    time_since_update text NOT NULL,
    index_36 text
);

CREATE TABLE level_data_meta (
  level_id bigint PRIMARY KEY NOT NULL,
  cached_at timestamp without time zone,
  absent boolean DEFAULT false NOT NULL
);

CREATE TABLE newgrounds_song (
    song_id bigint PRIMARY KEY NOT NULL,
    song_name text NOT NULL,
    index_3 bigint,
    song_artist text NOT NULL,
    filesize double precision NOT NULL,
    index_6 text,
    index_7 text,
    index_8 text,
    song_link text
);

CREATE TABLE newgrounds_song_meta (
    song_id bigint PRIMARY KEY NOT NULL,
    cached_at timestamp without time zone,
    absent boolean DEFAULT false NOT NULL
);

ALTER TABLE demons ADD COLUMN level_id INT8 NULL UNIQUE REFERENCES level(level_id);