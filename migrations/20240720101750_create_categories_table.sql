-- Add migration script here
CREATE TABLE categories (
    id              bigint          NOT NULL,
    created_at      bigint          NOT NULL,
    sort_key        int             NOT NULL,
    title           varchar(120)    NOT NULL,
    `order`         char(16)        NOT NULL,
    user_id         bigint          NOT NULL,
    track           tinyint(1)      NOT NULL,
    show_in_lib     tinyint(1)      NOT NULL,
    deleted_at      bigint          NOT NULL,

    PRIMARY KEY (id, user_id),

    CONSTRAINT  categories_ibfk_1
        FOREIGN KEY (user_id) REFERENCES users (id)
            ON DELETE CASCADE
);

CREATE INDEX categories_id_index
    ON categories (id);
