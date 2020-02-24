--DELETE FROM records AS r1
--WHERE r1.player = $1
--  AND r1.status_ = 'REJECTED'
--  AND EXISTS (SELECT 1 FROM records AS r2
--              WHERE r2.status_ = 'APPROVED'
--                AND r2.demon = r1.demon
--                AND r2.player = $1)
DELETE FROM records AS r1
USING records AS r2
WHERE r1.player = $1
  AND r1.status_ = 'REJECTED'
  AND r2.status_ = 'APPROVED'
  AND r2.demon = r1.demon
  AND r2.player = $1
RETURNING r2.id, r1.notes