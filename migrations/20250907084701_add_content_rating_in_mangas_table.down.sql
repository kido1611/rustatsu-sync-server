-- Add down migration script here
ALTER TABLE mangas
DROP COLUMN content_rating;
