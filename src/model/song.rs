use blake3::Hasher;
use diesel::prelude::*;
use lofty::{
    file::{AudioFile, TaggedFileExt},
    read_from,
    tag::{Accessor, ItemKey},
};
use std::{
    cmp::Ordering, ffi::OsStr, fs::File, io::{Read, Seek}, path::{Path, PathBuf}, time::Duration
};

use crate::model::Connection;
use crate::model::orm::*;
use crate::model::schema;

#[derive(Debug, Clone, Default, Eq)]
pub struct Song {
    pub title: String,
    pub length: Duration,
    pub path: PathBuf,
    pub track_number: i32,
    pub disc_number: i32,

    pub artists: Vec<Artist>,
    pub genres: Vec<Genre>,
    pub album: String,

    pub hash: Vec<u8>
}

macro_rules! fallthrough {
    ( $x:expr, $y:expr ) => {
        if ($x.cmp($y) != Ordering::Equal) {
            return Some($x.cmp($y));
        }
    }
}

impl PartialEq for Song {
    fn eq(&self, other: &Self) -> bool {
        self.hash == other.hash
    }
}

impl PartialOrd for Song {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        fallthrough!(self.album, &other.album);
        fallthrough!(self.disc_number, &other.disc_number);
        fallthrough!(self.track_number, &other.track_number);
        fallthrough!(self.title, &other.title);
        fallthrough!(self.hash, &other.hash);

        Some(Ordering::Equal)
    }
}

impl Ord for Song {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

macro_rules! field {
    ( $x:ident,$y:ident,$s:expr ) => {
        schema::$y::$x.eq(&$s.$x)
    };

    ( $x:ident,$y:ident ) => {
        schema::$y::$x.eq($x)
    };
}

impl Song {
    pub fn from_path(path: &Path) -> Result<Self, Error> {
        let mut file = File::open(path).map_err(|x| Error::IOError(x))?;
        let mut data = read_from(&mut file).map_err(|x| Error::MetadataError(x))?;

        let duration = data.properties().duration();

        let tag = match data.primary_tag_mut() {
            Some(primary_tag) => primary_tag,
            None => data.first_tag_mut().ok_or(Error::InvalidData).unwrap()
        };

        let mut artists = {
            tag.take_strings(&ItemKey::TrackArtists)
                .map(|x| Artist::from(x))
                .collect::<Vec<Artist>>()
        };

        artists.dedup();

        let genres = {
            tag.get_strings(&ItemKey::Genre)
                .flat_map(|x| x.split(','))
                .map(|x| Genre::from(x.to_string()))
                .collect::<Vec<Genre>>()
        };

        let title = if tag.title().is_some() {
            tag.title().map(|x| x.to_string()).unwrap_or(String::new())
        } else {
            path.file_stem()
                .ok_or(Error::IOError(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "could not get valid path",
                )))?
                .to_string_lossy()
                .to_string()
        };

        Ok(Song {
            path: path.to_path_buf(),
            title: title.clone(),

            track_number: tag.track().unwrap_or(1) as i32,
            length: duration,
            disc_number: tag.disk().unwrap_or(0) as i32,

            artists: {
                if artists.len() > 0 {
                    artists
                } else {
                    vec![Artist::from(
                        tag.artist()
                            .map(|x| x.to_string())
                            .unwrap_or(String::from("Unknown Artist")),
                    )]
                }
            },
            genres: {
                if genres.len() > 0 {
                    genres
                } else {
                    vec![Genre::from(
                        tag.genre()
                            .map(|x| x.to_string())
                            .unwrap_or(String::from("")),
                    )]
                }
            },
            album: tag
                .album()
                .map(|x| x.to_string())
                .unwrap_or(String::from(title)),

            hash: {
                file.rewind().map_err(|err| Error::IOError(err))?;

                let mut buf = [0; 128 * 1024];
                file.read(&mut buf).map_err(|err| Error::IOError(err))?;

                let mut hasher = Hasher::new();

                hasher.update(&buf);

                let length = std::fs::metadata(path).map_err(|err| Error::IOError(err))?.len() as i64;
                file.seek_relative(length / 2).map_err(|err| Error::IOError(err))?;

                file.read(&mut buf).map_err(|err| Error::IOError(err))?;
                hasher.update(&buf);
                hasher.update(&length.to_le_bytes());

                hasher.finalize().as_bytes().into()
            }
        })
    }

    pub fn from_db(song: &DbSong, conn: &mut Connection) -> Result<Song, diesel::result::Error> {
        Ok(Song {
            path: PathBuf::from(unsafe { OsStr::from_encoded_bytes_unchecked(&song.path) }),
            title: song.title.clone(),
            track_number: song.track_number,
            length: Duration::from_millis(song.length as u64),
            disc_number: song.disc_number,
            artists: SongArtist::belonging_to(&song)
                .inner_join(schema::artists::table)
                .select(_Artist::as_select())
                .load(conn)?
                .iter()
                .map(|x| x.into())
                .collect(),
            genres: SongGenre::belonging_to(&song)
                .inner_join(schema::genres::table)
                .select(_Genre::as_select())
                .load(conn)?
                .iter()
                .map(|x| x.into())
                .collect(),
            album: schema::albums::table
                .filter(schema::albums::album_id.eq(song.album_id))
                .select(schema::albums::name)
                .first(conn)?,
            hash: song.hash.clone()
        })
    }

    pub fn from_id(id: i32, conn: &mut Connection) -> Result<Self, diesel::result::Error> {
        let base = schema::songs::table
            .filter(schema::songs::song_id.eq(id))
            .select(DbSong::as_select())
            .first(conn)?;

        let genres = SongGenre::belonging_to(&base)
            .inner_join(schema::genres::table)
            .select(_Genre::as_select())
            .load(conn)?;

        let artists = SongArtist::belonging_to(&base)
            .inner_join(schema::artists::table)
            .select(_Artist::as_select())
            .load(conn)?;

        let album = schema::albums::table
            .filter(schema::albums::album_id.eq(base.album_id))
            .select(Album::as_select())
            .first(conn)?;

        let path = unsafe { OsStr::from_encoded_bytes_unchecked(&base.path) };

        Ok(Song {
            title: base.title,
            length: Duration::from_millis(base.length as u64),
            path: PathBuf::from(path),
            track_number: base.track_number,
            disc_number: base.disc_number,
            artists: artists.into_iter().map(|x| Artist::from(x.name)).collect(),
            genres: genres.into_iter().map(|x| Genre::from(x.name)).collect(),
            album: album.name,
            hash: base.hash,
        })
    }

    pub fn commit(&self, album: &Album, conn: &mut Connection) -> Result<(), diesel::result::Error> {
        let current = schema::songs::table
            .filter(schema::songs::hash.eq(&self.hash))
            .select(DbSong::as_select())
            .first(conn);

        if let Ok(song) = current {
            if song.path == self.path.as_os_str().as_encoded_bytes() {
                return Ok(());
            }

            return diesel::update(
                schema::songs::table.filter(
                    schema::songs::hash.eq(&self.hash)
                )
            )
                .set(schema::songs::path.eq(self.path.as_os_str().as_encoded_bytes()))
                .execute(conn).map(|_| ());
        }

        let song_id: i32 = diesel::insert_into(schema::songs::table)
            .values((
                field!(title, songs, self),
                field!(track_number, songs, self),
                field!(disc_number, songs, self),
                schema::songs::length.eq(self.length.as_millis() as i32),
                schema::songs::path.eq(self.path.clone().into_os_string().into_encoded_bytes()),
                schema::songs::album_id.eq(album.album_id as i32),
                field!(hash, songs, self)
            ))
            .returning(schema::songs::song_id)
            .get_result(conn)?;

        for artist in &self.artists {
            let artist_id: i32 = {
                let rout: Result<i32, diesel::result::Error> = schema::artists::table
                    .filter(schema::artists::name.eq(&artist.name))
                    .select(schema::artists::artist_id)
                    .first(conn);

                if matches!(rout, Err(diesel::result::Error::NotFound)) {
                    diesel::insert_into(schema::artists::table)
                        .values(schema::artists::name.eq(&artist.name))
                        .returning(schema::artists::artist_id)
                        .get_result(conn)
                } else {
                    rout
                }
            }?;

            diesel::insert_into(schema::song_artist::table)
                .values((field!(artist_id, song_artist), field!(song_id, song_artist)))
                .execute(conn)?;
        }

        for genre in &self.genres {
            let genre_id: i32 = {
                let rout: Result<i32, diesel::result::Error> = schema::genres::table
                    .filter(schema::genres::name.eq(&genre.name))
                    .select(schema::genres::genre_id)
                    .first(conn);

                if matches!(rout, Err(diesel::result::Error::NotFound)) {
                    diesel::insert_into(schema::genres::table)
                        .values(schema::genres::name.eq(&genre.name))
                        .returning(schema::genres::genre_id)
                        .get_result(conn)
                } else {
                    rout
                }
            }?;

            diesel::insert_into(schema::song_genre::table)
                .values((field!(genre_id, song_genre), field!(song_id, song_genre)))
                .execute(conn)?;
        }

        Ok(())
    }

    pub fn album_artists(&self) -> Result<Vec<String>, Error> {
        let mut file = File::open(&self.path).map_err(|x| Error::IOError(x))?;
        let mut data = read_from(&mut file).map_err(|x| Error::MetadataError(x))?;

        let tag = match data.primary_tag_mut() {
            Some(primary_tag) => primary_tag,
            None => data.first_tag_mut().ok_or(Error::InvalidData).unwrap()
        };

        Ok(tag.take_strings(&ItemKey::AlbumArtist).collect())
    }
}
