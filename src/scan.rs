use std::path::{Path, PathBuf};
use std::{io, thread};

use crate::Song;
use walkdir::WalkDir;

pub fn scan_directory(directory: &Path, threads: usize) -> io::Result<Vec<Song>> {
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

    Ok(thread::scope(|s| {
        let mut workers = Vec::with_capacity(threads);
        let files_chunked: Vec<&[PathBuf]> = files.chunks(chunk).collect();

        for i in 0..threads {
            let chunk = files_chunked[i];
            workers.push(s.spawn(move || {
                let mut data = Vec::new();

                for path in chunk {
                    let song = Song::from_path(path);

                    if song.is_ok() {
                        data.push(song.unwrap());
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
    }))
}

#[cfg(test)]
mod test {
    use std::path::Path;

    use super::scan_directory;

    #[test]
    fn test_scan() {
        let data = scan_directory(Path::new("/share/music/"), 4).unwrap();

        for item in data.iter().take(5) {
            println!("{}, {}", item.title, item.path.display());
        }
    }
}
