-- Your SQL goes here

CREATE OR REPLACE VIEW demon_publisher_verifier_join AS
SELECT p1.name AS pname, p1.id AS pid, p1.banned AS pbanned, p2.name AS vname, p2.id AS vid, p2.banned AS vbanned
FROM demons
INNER JOIN players AS p1
ON demons.publisher = p1.id
INNER JOIN players AS p2
ON demons.verifier = p2.id