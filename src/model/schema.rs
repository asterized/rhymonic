// @generated automatically by Diesel CLI.

diesel::table! {
    album_artist (album_id, artist_id) {
        album_id -> Integer,
        artist_id -> Integer,
    }
}

diesel::table! {
    albums (album_id) {
        album_id -> Integer,
        name -> Text,
    }
}

diesel::table! {
    artists (artist_id) {
        artist_id -> Integer,
        name -> Text,
    }
}

diesel::table! {
    genres (genre_id) {
        genre_id -> Integer,
        name -> Text,
        display_name -> Text,
    }
}

diesel::table! {
    song_artist (artist_id, song_id) {
        artist_id -> Integer,
        song_id -> Integer,
    }
}

diesel::table! {
    song_genre (genre_id, song_id) {
        genre_id -> Integer,
        song_id -> Integer,
    }
}

diesel::table! {
    songs (song_id) {
        song_id -> Integer,
        title -> Text,
        track_number -> Integer,
        length -> Integer,
        path -> Binary,
        disc_number -> Integer,
        album_id -> Integer,
    }
}

diesel::joinable!(album_artist -> albums (album_id));
diesel::joinable!(album_artist -> artists (artist_id));
diesel::joinable!(song_artist -> artists (artist_id));
diesel::joinable!(song_artist -> songs (song_id));
diesel::joinable!(song_genre -> genres (genre_id));
diesel::joinable!(song_genre -> songs (song_id));
diesel::joinable!(songs -> albums (album_id));

diesel::allow_tables_to_appear_in_same_query!(
    album_artist,
    albums,
    artists,
    genres,
    song_artist,
    song_genre,
    songs,
);
