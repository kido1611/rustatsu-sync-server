-- Add up migration script here
CREATE TABLE mangas (
    id              bigint          NOT NULL,
    title           varchar(255)    NOT NULL,
    alt_title       varchar(255)    NULL,
    url             varchar(255)    NOT NULL,
    public_url      varchar(255)    NOT NULL,
    rating          real            NOT NULL,
    is_nsfw         boolean         NOT NULL,
    cover_url       varchar(255)    NOT NULL,
    large_cover_url varchar(255)    NULL,
    state           varchar(24)     NULL,
    author          varchar(120)    NULL,
    source          varchar(120)    NOT NULL,

    PRIMARY KEY (id)
);
