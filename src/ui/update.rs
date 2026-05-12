use iced::Task;
use rfd::{AsyncFileDialog, FileHandle};
use std::{collections::HashSet, hash::Hash, sync::Arc};
use iced_runtime::task::blocking;

use crate::{
    App, MediaControl, MediaEvent, MediaSignal, Message, Song,
    model::{DbAlbum, UncommittedSong},
    scan::scan_directory,
};

fn identical<T: Eq + Hash>(
    first: impl Iterator<Item = T>,
    second: impl Iterator<Item = T>,
) -> bool {
    let found: HashSet<T> = HashSet::from_iter(first);

    for item in second {
        if found.contains(&item) {
            return true;
        }
    }

    false
}

impl App {
    fn play_song(&mut self, song: Arc<Song>) {
        let _ = self.channel.try_send(MediaSignal::PlaySong(song));
    }

    fn handle_event(&mut self, event: MediaEvent) {
        match event {
            MediaEvent::Connect((sender, position)) => {
                self.channel = sender;
                self.position = position;
                self.connected = true;
            }

            MediaEvent::Play => self.playing = true,
            MediaEvent::Pause => self.playing = false,

            MediaEvent::EndedSong | MediaEvent::FailedQueue => {
                self.queue_position += 1;
                self.play_song(self.queue[self.queue_position].clone());
                self.fill_queue();
            }
        }
    }

    pub fn handle_control(&mut self, control: MediaControl) {
        match control {
            MediaControl::Next => {
                self.queue_position += 1;
                self.play_song(self.queue[self.queue_position].clone());
            }

            MediaControl::Prev => {
                self.queue_position -= 1;
                self.play_song(self.queue[self.queue_position].clone());
                self.fill_queue();
            }

            MediaControl::Resume => {
                let _ = self.channel.try_send(MediaSignal::Play);
            }
            MediaControl::Pause => {
                let _ = self.channel.try_send(MediaSignal::Pause);
            }

            MediaControl::PlayPause => {
                let _ = self.channel.try_send(if self.playing {
                    MediaSignal::Pause
                } else {
                    MediaSignal::Play
                });
            }
        }
    }

    fn insert_song(&mut self, song: Arc<Song>) {
        let Err(location) = self.songs.binary_search(&song) else {
            return;
        };

        let album_artists = song.album_artists().unwrap_or_default();

        for album in self
            .albums
            .iter_mut()
            .filter(|album| album.name == song.album)
        {
            if album_artists == Vec::<String>::new()
                || identical(album_artists.iter(), album.artists.iter().map(|a| &a.name))
            {
                album.songs.push(song.clone());
                self.songs.insert(location, song);
                return;
            }
        }

        self.songs.insert(location, song);
    }

    fn import_song(&mut self, hndl: Option<FileHandle>) -> Option<()> {
        let handle = hndl?;
        let song = UncommittedSong::from_path(handle.path()).ok()?;
        let mut conn = self.pool.get().ok()?;

        if !song.album.is_empty() {
            let album = DbAlbum::search_name(&song.album, &mut conn)
                .unwrap_or(DbAlbum::new(&song.album, &song.hash, &mut conn).ok()?);

            self.insert_song(Arc::new(song.commit(&album, &mut conn).ok()?));
        }

        Some(())
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Media(event) => self.handle_event(event),
            Message::Control(control) => self.handle_control(control),

            Message::SetPosition(position) => {
                let _ = self.channel.try_send(MediaSignal::NewPosition(position));
            }

            Message::Queue(song) => self.queue.push(song.clone()),

            Message::Play(song) => {
                self.queue.clear();

                self.queue.push(song.clone());
                let _ = self.play_song(song.clone());
                self.queue_position = 0;

                self.fill_queue();
            }

            Message::SetPage(page) => self.page = page,
            Message::ScrollPosition(position) => self.scroll_position = position,

            Message::BeginImportSong => {
                let dialog = AsyncFileDialog::new();

                return Task::perform(dialog.pick_file(), |handle| Message::ImportSong(handle));
            }

            Message::BeginImportDir => {
                let dialog = AsyncFileDialog::new();

                return Task::perform(dialog.pick_folder(), Message::ImportDirectory);
            }

            Message::ImportSong(h) => {
                self.import_song(h);
            }

            Message::ImportDirectory(hndl) => {
                let Some(handle) = hndl else {
                    return Task::none();
                };

                let pool = self.pool.clone();

                return blocking(move |mut sender| {
                    let data = scan_directory(handle.path(), pool).unwrap_or(Vec::new());
                    let _ = sender.try_send(data);
                }).map(Message::DoneImport);
            }

            Message::DoneImport(albums) => {
                albums
                    .iter()
                    .flat_map(|album| album.songs.iter())
                    .for_each(|song| self.insert_song(song.clone()));

                self.albums.extend(albums);
            }
        };

        Task::none()
    }
}
