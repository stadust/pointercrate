/* ALTER TABLE IF EXISTS public.records
    ADD COLUMN enjoyment integer;

ALTER TABLE IF EXISTS public.records
    ALTER COLUMN enjoyment SET STORAGE PLAIN;

ALTER TABLE IF EXISTS public.rec_backup
    ADD COLUMN enjoyment integer;

ALTER TABLE IF EXISTS public.rec_backup
    ALTER COLUMN enjoyment SET STORAGE PLAIN;

 */