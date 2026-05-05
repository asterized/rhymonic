use crate::model::orm::objects::*;
use crate::model::schema;
use diesel::prelude::*;
use lofty::error::LoftyError;
use std::io;

#[derive(Identifiable, Selectable, Queryable, Associations, Debug)]
#[diesel(belongs_to(DbSong, foreign_key=song_id))]
#[diesel(belongs_to(_Genre, foreign_key=genre_id))]
#[diesel(table_name = schema::song_genre)]
#[diesel(primary_key(song_id, genre_id))]
pub struct SongGenre {
    song_id: i32,
    genre_id: i32,
}

#[derive(Identifiable, Selectable, Queryable, Associations, Debug)]
#[diesel(belongs_to(DbSong, foreign_key=song_id))]
#[diesel(belongs_to(_Artist, foreign_key=artist_id))]
#[diesel(table_name = schema::song_artist)]
#[diesel(primary_key(song_id, artist_id))]
pub struct SongArtist {
    song_id: i32,
    artist_id: i32,
}

#[derive(Identifiable, Selectable, Queryable, Associations, Debug)]
#[diesel(belongs_to(DbAlbum, foreign_key=album_id))]
#[diesel(belongs_to(_Artist, foreign_key=artist_id))]
#[diesel(table_name = schema::album_artist)]
#[diesel(primary_key(album_id, artist_id))]
pub struct AlbumArtist {
    album_id: i32,
    artist_id: i32,
}

#[derive(Identifiable, Selectable, Queryable, Associations, Debug)]
#[diesel(belongs_to(DbSong, foreign_key=song_id))]
#[diesel(belongs_to(_Playlist, foreign_key=playlist_id))]
#[diesel(table_name = schema::playlist_song)]
#[diesel(primary_key(playlist_id, song_id))]
pub struct PlaylistSong {
    playlist_id: i32,
    song_id: i32,
}

#[derive(Debug)]
pub enum Error {
    MetadataError(LoftyError),
    IOError(io::Error),
    InvalidData,
    NotFound,
}
