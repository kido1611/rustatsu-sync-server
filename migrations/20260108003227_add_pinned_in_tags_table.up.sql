-- Add up migration script here
BEGIN
;

ALTER TABLE
    tags
ADD
    COLUMN pinned boolean DEFAULT FALSE;

COMMIT;
