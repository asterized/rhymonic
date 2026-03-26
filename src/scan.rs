use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::io;

use crate::model::Artist;
use crate::{Album, Song};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use walkdir::WalkDir;

pub fn scan_directory(directory: &Path) -> io::Result<Vec<Album>> {
    let files: Vec<_> = WalkDir::new(directory)
        .into_iter()
        .filter_map(|e| e.ok())
        .map(|song| song.path().to_path_buf())
        .collect();

    let albums = files
        .into_par_iter()
        .filter_map(|path| Song::from_path(&path).ok())
        .map(Arc::new)
        .fold(HashMap::new, |mut map: HashMap<(String, Vec<_>), Vec<Arc<Song>>>, song| {
            map.entry(
                (
                    song.album.clone(),
                    song.album_artists().unwrap_or(Vec::new()))
                )
                .or_default().push(song);
            map
        })
        .reduce(HashMap::new, |mut a, b| {
            for (k, mut v) in b {
                a.entry(k).or_default().append(&mut v);
            }

            a
        });

    Ok(albums
        .into_iter()
        .map(|(key, items)| {
            let artists = {
                if key.1.is_empty() {
                    items.iter().fold(
                        Vec::with_capacity(items.len()),
                        |mut artists: Vec<_>, song| { artists.extend(song.artists.clone().into_iter()); artists }
                    )
                }
                else {
                    key.1.into_iter().map(|artist| Artist::from(artist)).collect()
                }
            };

            Album::new(key.0, items, artists)
        })
        .collect())
}
