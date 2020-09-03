-- This file should undo anything in `up.sql`

ALTER TABLE demons DROP COLUMN level_id;

DROP TABLE creator;
DROP TABLE creator_meta;

DROP TABLE level_data;
DROP TABLE level_data_meta;

DROP TABLE level;
DROP TABLE level_meta;
DROP TABLE level_request_results;
DROP TABLE level_request_meta;

DROP TABLE newgrounds_song;
DROP TABLE newgrounds_song_meta;