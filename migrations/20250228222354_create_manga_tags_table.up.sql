-- Add up migration script here
CREATE TABLE manga_tags(
    manga_id    bigint  NOT NULL,
    tag_id      bigint  NOT NULL,

    PRIMARY KEY (manga_id, tag_id),
    CONSTRAINT manga_tags_tag_id_foreign
        FOREIGN KEY (tag_id) REFERENCES tags (id),

    CONSTRAINT manga_tags_manga_id_foreign
        FOREIGN KEY (manga_id) REFERENCES mangas (id)
            ON DELETE CASCADE
);

CREATE INDEX manga_tags_tag_id_index
    ON manga_tags (tag_id);
