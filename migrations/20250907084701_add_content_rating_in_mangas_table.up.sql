-- Add up migration script here
ALTER TABLE mangas 
ADD COLUMN content_rating VARCHAR(100) NULL 
DEFAULT NULL;
