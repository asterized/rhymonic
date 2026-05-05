use std::collections::HashMap;
use std::io;
use std::path::Path;

use crate::Album;
use crate::model::{Artist, UncommittedSong};
use diesel::SqliteConnection;
use diesel::r2d2::ConnectionManager;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use walkdir::WalkDir;

pub fn scan_directory<'a>(
    directory: &Path,
    pool: diesel::r2d2::Pool<ConnectionManager<SqliteConnection>>,
) -> io::Result<Vec<Album>> {
    let files: Vec<_> = WalkDir::new(directory)
        .into_iter()
        .filter_map(|e| e.ok())
        .map(|song| song.path().to_path_buf())
        .collect();

    let albums = files
        .into_par_iter()
        .filter_map(|path| UncommittedSong::from_path(&path).ok())
        .fold(
            HashMap::new,
            |mut map: HashMap<(String, Vec<_>), Vec<UncommittedSong>>, song| {
                map.entry((
                    song.album.clone(),
                    song.album_artists().unwrap_or(Vec::new()),
                ))
                .or_default()
                .push(song);
                map
            },
        )
        .reduce(HashMap::new, |mut a, b| {
            for (k, mut v) in b {
                a.entry(k).or_default().append(&mut v);
            }

            a
        });

    Ok(albums
        .into_par_iter()
        .filter_map(|(key, items)| {
            let artists = {
                if key.1.is_empty() {
                    let mut song_artists = items.iter().fold(
                        Vec::with_capacity(items.len()),
                        |mut artists: Vec<_>, song| {
                            artists.extend(song.artists.clone().into_iter());
                            artists
                        },
                    );

                    song_artists.dedup();

                    song_artists
                } else {
                    key.1
                        .into_iter()
                        .map(|artist| Artist::from(artist))
                        .collect()
                }
            };

            Some(Album::from_uncommitted(key.0, items, artists, &mut *pool.get().ok()?).unwrap())
        })
        .collect())
}
