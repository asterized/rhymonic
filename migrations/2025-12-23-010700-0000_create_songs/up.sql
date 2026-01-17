-- Your SQL goes here

CREATE TABLE songs (
    song_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    track_number INTEGER NOT NULL,
    length INTEGER NOT NULL,
    path BLOB NOT NULL,
    disc_number INTEGER NOT NULL,
    album_id INTEGER UNIQUE NOT NULL REFERENCES albums(album_id)
);
