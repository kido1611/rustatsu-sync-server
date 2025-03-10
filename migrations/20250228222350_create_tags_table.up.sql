-- Add up migration script here
CREATE TABLE tags (
    id          bigint          NOT NULL,
    title       varchar(250)    NOT NULL,
    "key"       varchar(120)    NOT NULL,
    source      varchar(120)    NOT NULL,
    PRIMARY KEY (id)
);
