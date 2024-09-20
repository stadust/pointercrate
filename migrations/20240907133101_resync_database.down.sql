ALTER TABLE IF EXISTS public.demons
    ADD CONSTRAINT demons_level_id_fkey FOREIGN KEY (level_id)
    REFERENCES public.gj_level (level_id) MATCH SIMPLE
    ON UPDATE NO ACTION
    ON DELETE NO ACTION;



ALTER TABLE IF EXISTS public.records
    ADD CONSTRAINT records_video_key UNIQUE (video);
