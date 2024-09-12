-- This file should undo anything in `up.sql`
DROP VIEW demons_pv;
DROP VIEW demons_p;
DROP VIEW records_pds;
DROP VIEW records_pd;
DROP VIEW records_p;
DROP VIEW records_d;
DROP VIEW players_n;

CREATE OR REPLACE VIEW demon_verifier_publisher_join AS
SELECT p1.name AS vname, p1.id AS vid, p1.banned AS vbanned, p2.name AS pname, p2.id AS pid, p2.banned AS pbanned
FROM demons
INNER JOIN players AS p1
ON demons.verifier = p1.id
INNER JOIN players AS p2
ON demons.publisher = p2.id