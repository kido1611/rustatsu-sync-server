-- Add up migration script here
ALTER TABLE
    mangas
ALTER COLUMN
    title TYPE varchar(1024),
ALTER COLUMN
    alt_title TYPE varchar(1024),
ALTER COLUMN
    url TYPE varchar(1024),
ALTER COLUMN
    public_url TYPE varchar(1024),
ALTER COLUMN
    cover_url TYPE varchar(1024),
ALTER COLUMN
    large_cover_url TYPE varchar(1024);
