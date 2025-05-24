-- Add up migration script here

CREATE MATERIALIZED VIEW player_ranks AS
       SELECT
           RANK() OVER (ORDER BY score DESC) as rank,
           id
       FROM players
       WHERE
           score != 0;

CREATE UNIQUE INDEX player_ranks_id_idx ON player_ranks(id);