-- Add up migration script here
BEGIN
;

ALTER TABLE
    mangas
ADD
    COLUMN nsfw boolean DEFAULT FALSE;

UPDATE
    mangas
SET
    nsfw = (content_rating = 'ADULT');

UPDATE
    mangas
SET
    nsfw = FALSE
WHERE
    nsfw IS NULL;

ALTER TABLE
    mangas
ALTER COLUMN
    nsfw
SET
    NOT NULL;

COMMIT;
