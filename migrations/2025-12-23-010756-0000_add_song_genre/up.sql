-- Your SQL goes here

CREATE TABLE song_genre (
    genre_id INTEGER NOT NULL REFERENCES genres(genre_id),
    song_id INTEGER NOT NULL REFERENCES songs(song_id),

    PRIMARY KEY(genre_id, song_id)
);

CREATE INDEX song_genre_index ON song_genre(genre_id);
