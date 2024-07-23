-- Add migration script here
CREATE TABLE users (
    id                          int AUTO_INCREMENT PRIMARY KEY,
    email                       varchar(120) NOT NULL,
    password                    char(32) NOT NULL,
    nickname                    varchar(84) NULL,
    favourites_sync_timestamp   bigint  NULL,
    history_sync_timestamp      bigint  NULL
);

CREATE UNIQUE INDEX users_email_uindex ON users (email);
