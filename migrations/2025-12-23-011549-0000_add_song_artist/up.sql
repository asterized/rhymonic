-- Your SQL goes here

CREATE TABLE song_artist (
    artist_id INTEGER NOT NULL REFERENCES artists(artist_id),
    song_id INTEGER NOT NULL REFERENCES songs(song_id),

    PRIMARY KEY(artist_id, song_id)
);

CREATE INDEX song_artist_index ON song_artist (song_id);
CREATE INDEX artist_song_index ON song_artist (song_id);
