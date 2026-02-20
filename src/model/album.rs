use std::slice::Iter;
use std::sync::Arc;

use diesel::{ExpressionMethods, QueryDsl};
use diesel::{RunQueryDsl, SelectableHelper};

use crate::Song;
use crate::model::Connection;
use crate::model::orm::DbSong;
use crate::model::schema;

pub struct Album {
    songs: Vec<Arc<Song>>,
    name: String,
}

impl Album {
    pub fn from_id(id: i32, conn: &mut Connection) -> Result<Album, diesel::result::Error> {
        Ok(Self {
            songs: schema::songs::table
                .filter(schema::songs::album_id.eq(id))
                .select(DbSong::as_select())
                .load(conn)?
                .iter()
                .map(|x| Ok::<Arc<Song>, diesel::result::Error>(Arc::new(Song::from_db(x, conn)?)))
                .collect::<Result<Vec<_>, _>>()?,
            name: schema::albums::table
                .filter(schema::albums::album_id.eq(id))
                .select(schema::albums::name)
                .first(conn)?,
        })
    }

    pub fn from_vec(name: String, items: Vec<Arc<Song>>) -> Album {
        Self {
            songs: items,
            name: name,
        }
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
