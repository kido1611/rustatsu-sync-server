-- Add migration script here
CREATE TABLE favourites (
    manga_id        bigint      NOT NULL,
    category_id     bigint      NOT NULL,
    sort_key        int         NOT NULL,
    created_at      bigint      NOT NULL,
    deleted_at      bigint      NOT NULL,
    user_id         int         NOT NULL,

    PRIMARY KEY (manga_id, category_id, user_id),

    CONSTRAINT favourites_categories_id_pk
        FOREIGN KEY (category_id, user_id) REFERENCES categories (id, user_id),

    CONSTRAINT favourites_obfk_1
        FOREIGN KEY (manga_id) REFERENCES manga (id),

    CONSTRAINT favourites_ibfk_2
        FOREIGN KEY (user_id) REFERENCES users (id)
);

CREATE INDEX user_id
    ON favourites (user_id);
