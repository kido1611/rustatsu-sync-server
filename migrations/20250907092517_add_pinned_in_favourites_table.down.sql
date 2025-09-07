-- Add down migration script here
ALTER TABLE favourites
DROP COLUMN pinned;
