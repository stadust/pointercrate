-- Your SQL goes here

CREATE VIEW demons_pv AS  -- demons with publisher and verifier
    SELECT demons.position, demons.name, demons.requirement, demons.video,
           publishers.id AS publisher_id, publishers.name AS publisher_name, publishers.banned AS publisher_banned,
           verifiers.id AS verifier_id, verifiers.name AS verifier_name, verifiers.banned AS verifier_banned
    FROM demons
    INNER JOIN players AS verifiers
    ON verifiers.id = demons.verifier
    INNER JOIN players AS publishers
    ON publishers.id = demons.publisher;

CREATE VIEW demons_p AS  -- demons with publisher
    SELECT demons.position, demons.name, demons.video,
           publishers.id AS publisher_id, publishers.name AS publisher_name, publishers.banned AS publisher_banned
    FROM demons
    INNER JOIN players AS publishers
    ON publishers.id = demons.publisher;

-- for listed representation
CREATE VIEW records_pd AS  -- records with player and demon
    SELECT records.id, records.progress, records.video, records.status_, records.submitter AS submitter_id,
           players.id AS player_id, players.name AS player_name, players.banned AS player_banned,
           demons.name AS demon_name, demons.position
    FROM records
    INNER JOIN players
    ON records.player = players.id
    INNER JOIN demons
    ON demons.name = records.demon;

-- for full representation
CREATE VIEW records_pds AS  -- records with player, demon and submitter
    SELECT records_pd.id, records_pd.progress, records_pd.video, records_pd.status_,
           records_pd.player_id, records_pd.player_name, records_pd.player_banned,
           records_pd.demon_name, records_pd.position,
           submitters.submitter_id, submitters.banned AS submitter_banned
    FROM records_pd
    INNER JOIN submitters
    ON records_pd.submitter_id = submitters.submitter_id;

-- for minimal representation
CREATE VIEW records_p AS  -- records with player
    SELECT records.id, records.progress, records.video, records.status_, records.demon,
           players.id AS player_id, players.name AS player_name, players.banned AS player_banned
    FROM records
    INNER JOIN players
    ON records.player = players.id;

CREATE VIEW records_d AS  -- records with demon
    SELECT records.id, records.progress, records.video, records.status_, records.player,
           demons.name AS demon_name, demons.position
    FROM records
    INNER JOIN demons
    ON demons.name = records.demon;

CREATE VIEW players_n AS  -- players with nationality
    SELECT players.id, players.name, players.banned,
           nationalities.iso_country_code, nationalities.nation
    FROM players
    LEFT OUTER JOIN nationalities
    ON players.nationality = nationalities.iso_country_code;

DROP VIEW IF EXISTS demon_verifier_publisher_join;