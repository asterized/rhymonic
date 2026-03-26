-- Your SQL goes here

CREATE TABLE albums (
    album_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    hash BLOB NOT NULL
);

CREATE UNIQUE INDEX hash_albums_index ON albums(hash);
