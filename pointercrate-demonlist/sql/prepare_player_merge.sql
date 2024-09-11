--DELETE FROM records as r1
--WHERE r1.player IN ($1, $2)
--  AND EXISTS (
--         SELECT id FROM records as r2
--         WHERE r2.player IN ({0}, {1})
--           AND r2.id <> r1.id
--           -- If they both have the same progress, demon and status, it doesnt matter which one we delete
--           -- We choose the one with the smaller ID because why the fuck not
--           AND (r1.progress < r2.progress OR r1.progress = r2.progress AND r1.id < r2.id)
--           AND r2.status_ = r1.status_
--           AND r2.demon = r1.demon
 --    )
DELETE FROM records AS r1
USING records AS r2
WHERE r1.player IN ($1, $2)
  AND r1.id <> r2.id
  AND (r1.progress < r2.progress OR r1.progress = r2.progress AND r1.id < r2.id)
  AND r2.status_ = r1.status_
  AND r2.demon = r1.demon
-- RETURNING CASE WHEN r1.notes IS NOT NULL THEN (r2.id, r1.notes) END TODO: figure out how to implement this
RETURNING r2.id, r1.notes