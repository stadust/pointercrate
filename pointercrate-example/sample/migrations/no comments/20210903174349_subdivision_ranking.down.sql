-- This file should undo anything in `up.sql`

DROP FUNCTION subdivision_ranking_of(country varchar(2));
DROP FUNCTION best_records_local(country VARCHAR(2), the_subdivision VARCHAR(3));