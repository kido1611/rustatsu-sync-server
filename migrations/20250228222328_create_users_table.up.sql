-- Add up migration script here
CREATE TABLE users (
    id                          BIGSERIAL PRIMARY KEY,
    email                       varchar(120) NOT NULL,
    password                    varchar(255) NOT NULL,
    nickname                    varchar(120) NULL,
    favourites_sync_timestamp   bigint  NULL,
    history_sync_timestamp      bigint  NULL
);

CREATE UNIQUE INDEX users_email_index ON users (email);
