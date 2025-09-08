-- Add up migration script here
UPDATE mangas
SET content_rating = 'ADULT'
WHERE is_nsfw = true;
