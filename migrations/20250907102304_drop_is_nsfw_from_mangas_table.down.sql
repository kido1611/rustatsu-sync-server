-- Add down migration script here
ALTER TABLE mangas
ADD COLUMN is_nsfw BOOLEAN NOT NULL
DEFAULT false;

UPDATE mangas
SET is_nsfw = true
WHERE content_rating = 'ADULT';
