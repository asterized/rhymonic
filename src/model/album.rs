use std::hash::Hash;
use std::slice::Iter;
use std::sync::Arc;

use diesel::prelude::*;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::model::orm::{_Artist, AlbumArtist, Artist, DbAlbum, DbSong};
use crate::model::{Connection, DatabaseError};
use crate::model::{Hashed, UncommittedSong, combine, schema};
use crate::{ConnectionPool, Song};

fn hash_songs<T: Hashed>(songs: &Vec<T>) -> [u8; 32] {
    let mut output = [0u8; 32];

    for song in songs.iter() {
        output = combine(&output, song.get_hash().try_into().unwrap());
    }

    output
}

#[derive(Debug, Clone)]
pub struct Album {
    id: i32,
    pub songs: Vec<Arc<Song>>,
    pub name: String,
    pub artists: Vec<Artist>,
    pub hash: [u8; 32],
}

impl PartialEq for Album {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Album {}

impl Hash for Album {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.hash.hash(state);
    }
}

impl Album {
    pub fn from_db(album: &DbAlbum, conn: &mut Connection) -> Result<Self, diesel::result::Error> {
        Ok(Self {
            id: album.album_id,
            songs: DbSong::belonging_to(&album)
                .select(DbSong::as_select())
                .load(conn)?
                .iter()
                .filter_map(|song| Song::from_db(song, conn).ok())
                .map(|s| Arc::new(s))
                .collect(),
            artists: AlbumArtist::belonging_to(&album)
                .inner_join(schema::artists::table)
                .select(_Artist::as_select())
                .load(conn)?
                .iter()
                .map(|artist| artist.into())
                .collect(),
            name: album.name.clone(),
            hash: album
                .hash
                .clone()
                .try_into()
                .expect("if this fails, something went very wrong"),
        })
    }

    pub fn from_id(id: i32, conn: &mut Connection) -> Result<Self, diesel::result::Error> {
        let album: DbAlbum = schema::albums::table
            .filter(schema::albums::album_id.eq(id))
            .first(conn)?;

        Album::from_db(&album, conn)
    }

    pub fn load(conn: &mut ConnectionPool) -> Result<Vec<Self>, DatabaseError> {
        Ok(schema::albums::table
            .load(&mut conn.get()?)?
            .par_iter()
            .filter_map(|album: &DbAlbum| {
                let mut connection = conn.get().ok()?;
                Some(Album::from_db(album, &mut connection).ok()?)
            })
            .collect())
    }

    pub fn from_uncommitted(
        name: String,
        songs: Vec<UncommittedSong>,
        artists: Vec<Artist>,
        conn: &mut SqliteConnection,
    ) -> Result<Self, diesel::result::Error> {
        let hash = hash_songs(&songs);
        let raw = DbAlbum::new(&name, &hash, conn)?;

        let committed = songs
            .iter()
            .filter_map(|song| song.commit(&raw, conn).ok())
            .map(Arc::new)
            .collect();

        Ok(Self {
            id: raw.album_id,
            name: name,
            songs: committed,
            artists: artists,
            hash: hash,
        })
    }

    pub fn iter(&self) -> Iter<'_, Arc<Song>> {
        self.songs.iter()
    }
}

impl IntoIterator for Album {
    type Item = Arc<Song>;
    type IntoIter = <Vec<Self::Item> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.songs.into_iter()
    }
}
