-- Add up migration script here
CREATE TABLE history(
    manga_id    bigint      NOT NULL,
    created_at  bigint      NOT NULL,
    updated_at  bigint      NOT NULL,
    chapter_id  bigint      NOT NULL,
    page        smallint    NOT NULL,
    scroll      real        NOT NULL,
    percent     real        NOT NULL,
    chapters    int         NOT NULL,
    deleted_at  bigint      NOT NULL,
    user_id     integer     NOT NULL,

    PRIMARY KEY (user_id, manga_id),

    CONSTRAINT history_manga_id_foreign
        FOREIGN KEY (manga_id) REFERENCES mangas(id),

    CONSTRAINT history_user_id_foreign
        FOREIGN KEY (user_id) REFERENCES users (id)
            ON DELETE CASCADE
);

CREATE INDEX history_manga_id_index
    ON history (manga_id);
