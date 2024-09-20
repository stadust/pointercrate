CREATE TABLE public.rec_backup
(
    id integer DEFAULT nextval('records_id_seq'::regclass),
    progress smallint,
    video character varying(200),
    status_ record_status,
    player integer,
    submitter integer,
    demon integer,
    raw_footage text,
    demon_name citext,
    PRIMARY KEY (id)
)

USING heap
TABLESPACE pg_default;

ALTER TABLE IF EXISTS public.rec_backup
    OWNER to pointercrate;