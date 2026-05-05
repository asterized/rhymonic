use crate::model::Connection;
use crate::model::schema;
use diesel::prelude::*;

#[derive(Debug, Clone, Queryable, Selectable, Identifiable)]
#[diesel(primary_key(album_id))]
#[diesel(table_name = schema::albums)]
pub struct DbAlbum {
    pub album_id: i32,
    pub name: String,
    pub hash: Vec<u8>,
}

impl DbAlbum {
    pub fn new(
        name: &String,
        hash: &[u8],
        conn: &mut Connection,
    ) -> Result<Self, diesel::result::Error> {
        if let Ok(album) = schema::albums::table
            .filter(schema::albums::hash.eq(hash))
            .first(conn)
        {
            return Ok(album);
        }

        let album = diesel::insert_into(schema::albums::table)
            .values((schema::albums::name.eq(name), schema::albums::hash.eq(hash)))
            .get_result(conn)?;

        Ok(album)
    }

    pub fn search_name(name: &String, conn: &mut Connection) -> Option<Self> {
        schema::albums::table
            .filter(schema::albums::name.eq(name))
            .first(conn)
            .ok()
    }
}

#[derive(Debug, Clone, Queryable, Selectable, Identifiable)]
#[diesel(primary_key(artist_id))]
#[diesel(table_name = schema::artists)]
pub struct _Artist {
    pub artist_id: i32,
    pub name: String,
}

impl _Artist {
    pub fn search_exact(name: &str, conn: &mut Connection) -> Result<Self, diesel::result::Error> {
        schema::artists::table
            .filter(schema::artists::name.eq(name))
            .select(Self::as_select())
            .first(conn)
    }

    pub fn insert(name: &str, conn: &mut Connection) -> Result<usize, diesel::result::Error> {
        diesel::insert_into(schema::artists::table)
            .values(schema::artists::name.eq(name))
            .returning(Self::as_returning())
            .execute(conn)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Artist {
    pub name: String,
    committed: bool,
}

impl Artist {
    pub fn commit(&mut self, conn: &mut Connection) -> Result<(), diesel::result::Error> {
        if self.committed {
            return Ok(());
        }

        let insertion = _Artist::insert(&self.name, conn);

        match insertion {
            Ok(_) => Ok(()),
            Err(diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UniqueViolation,
                _,
            )) => {
                self.committed = true;
                Ok(())
            }
            Err(x) => Err(x),
        }
    }
}

impl From<String> for Artist {
    fn from(value: String) -> Self {
        Artist {
            name: value,
            committed: false,
        }
    }
}

impl From<&_Artist> for Artist {
    fn from(value: &_Artist) -> Self {
        Artist {
            name: value.name.clone(),
            committed: true,
        }
    }
}

#[derive(Identifiable, Selectable, Queryable, Associations, Debug)]
#[diesel(belongs_to(DbAlbum, foreign_key=album_id))]
#[diesel(table_name = schema::songs)]
#[diesel(primary_key(song_id))]
pub struct DbSong {
    pub song_id: i32,
    pub title: String,
    pub track_number: i32,
    pub length: i32,
    pub path: Vec<u8>,
    pub disc_number: i32,
    pub album_id: i32,
    pub hash: Vec<u8>,
}

#[derive(Debug, Clone, Queryable, Selectable, Identifiable)]
#[diesel(table_name = schema::genres)]
#[diesel(primary_key(genre_id))]
pub struct _Genre {
    genre_id: i32,
    pub name: String,
}

impl _Genre {
    pub fn search_exact(name: &str, conn: &mut Connection) -> Result<Self, diesel::result::Error> {
        schema::genres::table
            .filter(schema::genres::name.eq(name))
            .select(Self::as_select())
            .first(conn)
    }

    pub fn insert(name: &str, conn: &mut Connection) -> Result<usize, diesel::result::Error> {
        diesel::insert_into(schema::genres::table)
            .values(schema::genres::name.eq(name))
            .returning(Self::as_returning())
            .execute(conn)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Genre {
    pub name: String,
    committed: bool,
}

impl Genre {
    pub fn commit(&mut self, conn: &mut Connection) -> Result<(), diesel::result::Error> {
        if self.committed {
            return Ok(());
        }

        let insertion = _Genre::insert(&self.name, conn);

        match insertion {
            Ok(_) => Ok(()),
            Err(diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UniqueViolation,
                _,
            )) => {
                self.committed = true;
                Ok(())
            }
            Err(x) => Err(x),
        }
    }
}

impl From<String> for Genre {
    fn from(value: String) -> Self {
        Genre {
            name: value,
            committed: false,
        }
    }
}

impl From<&_Genre> for Genre {
    fn from(value: &_Genre) -> Self {
        Genre {
            name: value.name.clone(),
            committed: true,
        }
    }
}

#[derive(Queryable, Selectable, Identifiable, Debug, Clone)]
#[diesel(table_name = schema::playlists)]
#[diesel(primary_key(playlist_id))]
pub struct _Playlist {
    playlist_id: i32,
    name: String,
}
