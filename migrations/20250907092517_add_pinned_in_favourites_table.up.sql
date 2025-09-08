-- Add up migration script here
ALTER TABLE favourites
ADD COLUMN pinned BOOLEAN NOT NULL
DEFAULT false;
