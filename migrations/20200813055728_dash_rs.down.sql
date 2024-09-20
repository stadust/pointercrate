-- This file should undo anything in `up.sql`

ALTER TABLE demons DROP COLUMN level_id;

DROP TABLE gj_creator;
DROP TABLE gj_creator_meta;

DROP TABLE gj_level_data;
DROP TABLE gj_level_data_meta;

DROP TABLE gj_level;
DROP TABLE gj_level_meta;
DROP TABLE gj_level_request_results;
DROP TABLE gj_level_request_meta;

DROP TABLE gj_newgrounds_song;
DROP TABLE gj_newgrounds_song_meta;

DROP TABLE download_lock;