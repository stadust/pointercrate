-- Add down migration script here

CREATE TABLE gj_creator_meta (
    user_id bigint PRIMARY KEY NOT NULL,  -- No REFERENCES creator(user_id) as we also have to keep track of _missing_ entries here!
    cached_at timestamp without time zone NOT NULL,
    absent boolean DEFAULT false NOT NULL
);

CREATE TABLE gj_level_meta (
    level_id bigint PRIMARY KEY NOT NULL,
    cached_at timestamp without time zone NOT NULL,
    absent boolean DEFAULT false NOT NULL
);

CREATE TABLE gj_level_request_results (
    level_id bigint NOT NULL,
    request_hash bigint NOT NULL
);

CREATE TABLE gj_level_request_meta (
    request_hash bigint PRIMARY KEY NOT NULL,
    cached_at timestamp without time zone NOT NULL,
    absent boolean DEFAULT false NOT NULL
);


CREATE TABLE gj_level_data_meta (
  level_id bigint PRIMARY KEY NOT NULL,
  cached_at timestamp without time zone NOT NULL,
  absent boolean DEFAULT false NOT NULL
);

CREATE TABLE gj_newgrounds_song_meta (
    song_id bigint PRIMARY KEY NOT NULL,
    cached_at timestamp without time zone NOT NULL,
    absent boolean DEFAULT false NOT NULL
);


CREATE TABLE download_lock(
    level_id bigint not null
);