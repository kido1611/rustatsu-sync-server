-- Add down migration script here
ALTER TABLE
    tags DROP COLUMN pinned;
