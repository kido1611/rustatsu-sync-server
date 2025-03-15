-- Add up migration script here
CREATE TABLE favourites (
    manga_id        bigint      NOT NULL,
    category_id     bigint      NOT NULL,
    user_id         bigint      NOT NULL,
    sort_key        int         NOT NULL,
    created_at      bigint      NOT NULL,
    deleted_at      bigint      NOT NULL,

    PRIMARY KEY (manga_id, category_id, user_id),

    CONSTRAINT favourites_category_id_user_id_foreign
        FOREIGN KEY (category_id, user_id) REFERENCES categories (id, user_id),

    CONSTRAINT favourites_manga_id_foreign
        FOREIGN KEY (manga_id) REFERENCES mangas (id),

    CONSTRAINT favourites_user_id_foreign
        FOREIGN KEY (user_id) REFERENCES users (id)
);

CREATE INDEX favourites_user_id_index
    ON favourites (user_id);
