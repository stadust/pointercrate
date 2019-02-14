DELETE FROM records as r1
WHERE r1.player = {0}
  AND EXISTS (
         SELECT id FROM records as r2
         WHERE r2.player = {0}
           AND r2.id <> r1.id
           AND (r1.progress < r2.progress or r1.progress = r2.progress AND r1.status_ = 'REJECTED')
           AND r1.demon = r2.demon
     )