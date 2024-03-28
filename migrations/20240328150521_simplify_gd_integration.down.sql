-- Add down migration script here

CREATE TABLE download_lock(
    level_id bigint not null
);