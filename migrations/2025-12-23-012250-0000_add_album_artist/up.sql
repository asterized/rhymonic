-- Your SQL goes here

CREATE TABLE album_artist (
    album_id INTEGER NOT NULL REFERENCES albums(album_id),
    artist_id INTEGER NOT NULL REFERENCES artists(artist_id),

    PRIMARY KEY(album_id, artist_id)
);

CREATE INDEX album_artist_index ON album_artist (album_id);
CREATE INDEX artist_album_index ON album_artist (album_id);
