use std::slice::Iter;
use std::sync::Arc;

use blake3::Hasher;
use diesel::{Connection as _, prelude::*};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::{Song, ConnectionPool};
use crate::model::{Connection, DatabaseError};
use crate::model::orm::{_Artist, AlbumArtist, Artist, DbSong, Album as DbAlbum};
use crate::model::schema;

pub struct Album {
    songs: Vec<Arc<Song>>,
    name: String,
    artists: Vec<Artist>,
    hash: Vec<u8>
}

impl Album {
    pub fn from_db(album: &DbAlbum, conn: &mut Connection) -> Result<Self, diesel::result::Error> {
        Ok(Self {
            songs: DbSong::belonging_to(&album)
                .select(DbSong::as_select())
                .load(conn)?
                .iter().filter_map(|song| Song::from_db(song, conn).ok())
                .map(|s| Arc::new(s))
                .collect(),
            artists: AlbumArtist::belonging_to(&album)
                .inner_join(schema::artists::table)
                .select(_Artist::as_select())
                .load(conn)?
                .iter().map(|artist| artist.into())
                .collect(),
            hash: album.hash.clone(),
            name: album.name.clone()
        })
    }

    pub fn from_id(id: i32, conn: &mut Connection) -> Result<Self, diesel::result::Error> {
        let album: DbAlbum = schema::albums::table
            .filter(schema::albums::album_id.eq(id))
            .first(conn)?;

        Album::from_db(&album, conn)
    }

    pub fn load(conn: &mut ConnectionPool) -> Result<Vec<Self>, DatabaseError> {
        Ok(
            schema::albums::table.load(&mut conn.get()?)?
                .par_iter()
                .filter_map(|album: &DbAlbum| {
                    let mut connection = conn.get().ok()?;
                    Some(Album::from_db(album, &mut connection).ok()?)
                })
                .collect()
        )
    }

    pub fn new(name: String, items: Vec<Arc<Song>>, artists: Vec<Artist>) -> Album {
        Self {
            name: name,
            artists: artists,
            hash: {
                let mut hasher = Hasher::new();

                for song in items.iter() {
                    hasher.update(&song.hash);
                }

                hasher.finalize().as_bytes().to_vec()
            },

            songs: items,
        }
    }

    pub fn iter(&self) -> Iter<'_, Arc<Song>> {
        self.songs.iter()
    }

    pub fn commit(&self, conn: &mut Connection) -> Result<DbAlbum, diesel::result::Error> {
        if let Ok(album) = schema::albums::table
            .filter(schema::albums::hash.eq(&self.hash))
            .first(conn) {
                return Ok(album);
        }

        let album = diesel::insert_into(schema::albums::table)
            .values((schema::albums::name.eq(&self.name), schema::albums::hash.eq(&self.hash)))
            .get_result(conn)?;

        for song in self.songs.iter() {
            let _ = conn.transaction(|transaction| Ok::<_, diesel::result::Error>(song.commit(&album, transaction).unwrap()));
        }

        Ok(album)
    }
}

impl IntoIterator for Album {
    type Item = Arc<Song>;
    type IntoIter = <Vec<Self::Item> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.songs.into_iter()
    }
}
