-- Add up migration script here

CREATE MATERIALIZED VIEW player_ranks AS
       SELECT
           RANK() OVER (ORDER BY score DESC) as rank,
           id
       FROM players
       WHERE
           score != 0 AND NOT banned;

CREATE UNIQUE INDEX player_ranks_id_idx ON player_ranks(id);


CREATE OR REPLACE VIEW ranked_players AS
SELECT
    ROW_NUMBER() OVER(ORDER BY rank, id) AS index,
    rank,
    id, name, players.score, subdivision,
    nationalities.iso_country_code,
    nationalities.nation,
    nationalities.continent
FROM players
LEFT OUTER JOIN nationalities
             ON players.nationality = nationalities.iso_country_code
NATURAL JOIN player_ranks;