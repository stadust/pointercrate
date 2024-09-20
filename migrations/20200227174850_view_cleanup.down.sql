-- This file should undo anything in `up.sql`


CREATE VIEW records_pds AS  -- records with player, demon and submitter
    SELECT records.id, records.progress, records.video, records.status_, records.notes,
           players.id AS player_id, players.name AS player_name, players.banned AS player_banned,
           demons.id AS demon_id, demons.name AS demon_name, demons.position,
           submitters.submitter_id, submitters.banned AS submitter_banned
    FROM records
    INNER JOIN submitters
    ON records.submitter = submitters.submitter_id
    INNER JOIN players
    ON records.player = players.id
    INNER JOIN demons
    ON demons.id = records.demon;

CREATE VIEW records_pd AS  -- records with player and demon
    SELECT records.id, records.progress, records.video, records.status_, records.submitter AS submitter_id,
           players.id AS player_id, players.name AS player_name, players.banned AS player_banned,
           demons.id AS demon_id, demons.name AS demon_name, demons.position
    FROM records
    INNER JOIN players
    ON records.player = players.id
    INNER JOIN demons
    ON demons.id = records.demon;

-- for minimal representation
CREATE VIEW records_p AS  -- records with player
    SELECT records.id, records.progress, records.video, records.status_, records.demon,
           players.id AS player_id, players.name AS player_name, players.banned AS player_banned
    FROM records
    INNER JOIN players
    ON records.player = players.id;

CREATE VIEW records_d AS  -- records with demon
    SELECT records.id, records.progress, records.video, records.status_, records.player,
           demons.id AS demon_id, demons.name AS demon_name, demons.position
    FROM records
    INNER JOIN demons
    ON demons.id = records.demon;


CREATE VIEW demons_pv AS  -- demons with publisher and verifier
    SELECT demons.id, demons.position, demons.name, demons.requirement, demons.video,
           publishers.id AS publisher_id, publishers.name AS publisher_name, publishers.banned AS publisher_banned,
           verifiers.id AS verifier_id, verifiers.name AS verifier_name, verifiers.banned AS verifier_banned
    FROM demons
    INNER JOIN players AS verifiers
    ON verifiers.id = demons.verifier
    INNER JOIN players AS publishers
    ON publishers.id = demons.publisher;

CREATE VIEW demons_p AS  -- demons with publisher
    SELECT demons.id, demons.position, demons.name, demons.video,
           publishers.id AS publisher_id, publishers.name AS publisher_name, publishers.banned AS publisher_banned
    FROM demons
    INNER JOIN players AS publishers
    ON publishers.id = demons.publisher;


CREATE VIEW players_n AS  -- players with nationality
    SELECT players.id, players.name, players.banned,
           nationalities.iso_country_code, nationalities.nation
    FROM players
    LEFT OUTER JOIN nationalities
    ON players.nationality = nationalities.iso_country_code;