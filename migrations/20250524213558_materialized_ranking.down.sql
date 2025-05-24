-- Add down migration script here

CREATE OR REPLACE VIEW ranked_players AS
SELECT
    ROW_NUMBER() OVER(ORDER BY players.score DESC, id) AS index,
    RANK() OVER(ORDER BY players.score DESC) AS rank,
    id, name, players.score, subdivision,
    nationalities.iso_country_code,
    nationalities.nation,
    nationalities.continent
FROM players
         LEFT OUTER JOIN nationalities
                         ON players.nationality = nationalities.iso_country_code
WHERE NOT players.banned AND players.score > 0.0;

DROP MATERIALIZED VIEW player_ranks;