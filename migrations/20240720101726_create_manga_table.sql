-- Add migration script here

CREATE TABLE manga (
    id              bigint          NOT NULL,
    title           varchar(84)     NOT NULL,
    alt_title       varchar(84)     NULL,
    url             varchar(255)    NOT NULL,
    public_url      varchar(255)    NOT NULL,
    rating          float           NOT NULL,
    is_nsfw         tinyint(1)      NOT NULL,
    cover_url       varchar(255)    NOT NULL,
    large_cover_url varchar(255)    NULL,
    state           char(24)        NULL,
    author          varchar(32)     NULL,
    source          varchar(32)     NOT NULL,

    PRIMARY KEY (id)
);
