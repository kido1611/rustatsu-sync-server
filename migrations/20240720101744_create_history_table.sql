-- Add migration script here
CREATE TABLE history(
    manga_id    bigint      NOT NULL,
    created_at  bigint      NOT NULL,
    updated_at  bigint      NOT NULL,
    chapter_id  bigint      NOT NULL,
    page        smallint    NOT NULL,
    scroll      double      NOT NULL,
    percent     double      NOT NULL,
    chapters    int         NOT NULL,
    deleted_at  bigint      NOT NULL,
    user_id     int         NOT NULL,

    PRIMARY KEY (user_id, manga_id),

    CONSTRAINT history_ibfk_1
        FOREIGN KEY (manga_id) REFERENCES manga(id),

    CONSTRAINT history_ibfk_2
        FOREIGN KEY (user_id) REFERENCES users (id)
            ON DELETE CASCADE
);

CREATE INDEX manga_id
    ON history (manga_id);
