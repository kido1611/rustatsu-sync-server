-- Add up migration script here
CREATE TABLE categories (
    id              bigint          NOT NULL,
    created_at      bigint          NOT NULL,
    sort_key        int             NOT NULL,
    title           varchar(255)    NOT NULL,
    "order"         varchar(16)     NOT NULL,
    user_id         bigint          NOT NULL,
    track           boolean         NOT NULL,
    show_in_lib     boolean         NOT NULL,
    deleted_at      bigint          NOT NULL,

    PRIMARY KEY (id, user_id),

    CONSTRAINT  categories_user_id_foreign
        FOREIGN KEY (user_id) REFERENCES users (id)
            ON DELETE CASCADE
);

CREATE INDEX categories_id_index
    ON categories (id);
