-- Your SQL goes here

CREATE TABLE genres (
    genre_id INTEGER NOT NULL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE
);

CREATE UNIQUE INDEX genre_name_index ON genres(name);
