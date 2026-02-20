use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::{io, thread};

use crate::{Album, Song};
use walkdir::WalkDir;

pub fn scan_directory(directory: &Path, threads: usize) -> io::Result<Vec<Album>> {
    let mut files = Vec::new();

    for entry in WalkDir::new(directory).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        files.push(path.to_path_buf());
    }
    let size = files.len();
    let mut chunk = size / threads;
    if size % threads > 0 {
        chunk += 1;
    }

    let songs = thread::scope(|s| {
        let mut workers = Vec::with_capacity(threads);
        let files_chunked: Vec<&[PathBuf]> = files.chunks(chunk).collect();

        for i in 0..threads {
            let chunk = files_chunked[i];
            workers.push(s.spawn(move || {
                let mut data = Vec::new();

                for path in chunk {
                    let song = Song::from_path(path);

                    if let Ok(s) = song {
                        data.push(Arc::new(s));
                    }
                }

                data
            }));
        }

        let mut output = Vec::new();
        for worker in workers {
            output.extend(worker.join().unwrap());
        }

        output
    });

    let mut albums: HashMap<String, Vec<Arc<Song>>> = HashMap::with_capacity(songs.len());
    for song in songs {
        let album = {
            if let Some(x) = albums.get_mut(&song.album) {
                x
            } else {
                albums.insert(song.album.clone(), Vec::new());
                albums.get_mut(&song.album).unwrap()
            }
        };

        album.push(song);
    }

    Ok(albums
        .into_iter()
        .map(|(name, items)| Album::from_vec(name, items))
        .collect())
}
