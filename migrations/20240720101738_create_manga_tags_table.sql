-- Add migration script here
CREATE TABLE manga_tags(
    manga_id    bigint  NOT NULL,
    tag_id      bigint  NOT NULL,

    PRIMARY KEY (manga_id, tag_id),
    CONSTRAINT manga_tags_ibfk_1
        FOREIGN KEY (tag_id) REFERENCES tags (id),

    CONSTRAINT manga_tags_ibfk_2
        FOREIGN KEY (manga_id) REFERENCES manga (id)
            ON DELETE CASCADE
);

CREATE INDEX tag_id
    ON manga_tags (tag_id);
