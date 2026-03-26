-- Your SQL goes here

CREATE TABLE playlist_song (
    playlist_id INTEGER NOT NULL REFERENCES playlist(playlist_id),
    song_id INTEGER NOT NULL REFERENCES songs(song_id)
);
