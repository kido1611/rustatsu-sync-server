-- Add migration script here
CREATE TABLE tags (
    id          bigint          NOT NULL,
    title       varchar(64)     NOT NULL,
    `key`       varchar(120)    NOT NULL,
    source      varchar(32)     NOT NULL,
    PRIMARY KEY (id)
);
