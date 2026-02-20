use directories::ProjectDirs;
use iced::futures::channel::mpsc;
use iced::{Subscription, Theme};

use std::path::Path;
use std::sync::Arc;
use std::thread::available_parallelism;

use diesel::{Connection, SqliteConnection};

pub mod ellipsize;
mod listener;
pub mod model;
mod scan;
mod ui;
pub use model::Album;
pub use model::Song;

use crate::listener::listen;
use crate::scan::scan_directory;

pub fn main() -> iced::Result {
    iced::application(App::new, App::update, App::view)
        .subscription(App::subscription)
        .theme(App::theme)
        .run()
}

#[derive(Debug, Clone)]
pub enum Message {
    Queue(Arc<Song>),
    Play(Arc<Song>),
    Media(MediaEvent),
    Send(MediaSignal),
}

#[derive(Debug, Clone)]
pub enum MediaEvent {
    Connect(mpsc::Sender<MediaSignal>),
    EndedSong,
    FailedQueue,
    Queued,
    Play,
    Pause,
}

#[derive(Debug, Clone)]
pub enum MediaSignal {
    PlaySong(Arc<Song>),
    AddSong(Arc<Song>),
    PlayPause,
    Next,
}

pub struct App {
    theme: Theme,
    conn: Option<SqliteConnection>,
    albums: Vec<Album>,

    queue: Vec<Arc<Song>>,
    queue_position: usize,

    volume: f32,

    channel: mpsc::Sender<MediaSignal>,
    connected: bool,
    playing: bool,
}

impl App {
    fn new() -> Self {
        let location = ProjectDirs::from("io.github", "asterized", "rhythmic");
        let threads = available_parallelism().map(|x| x.get()).unwrap_or(4);
        let data = scan_directory(Path::new("/share/music/"), threads).unwrap();

        let application = Self {
            theme: Theme::Dark,

            conn: {
                if location.is_some() {
                    let db_location = location.as_ref().unwrap().data_local_dir().join("songs.db");
                    let conn = SqliteConnection::establish(
                        &db_location.into_os_string().to_string_lossy(),
                    );
                    conn.ok()
                } else {
                    None
                }
            },

            albums: data,

            queue: Vec::new(),
            queue_position: 0,

            volume: 50.0,

            channel: mpsc::channel(0).0,
            connected: false,
            playing: false,
        };

        application
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::run(listen).map(Message::Media)
    }

    fn theme(&self) -> Theme {
        self.theme.clone()
    }
}
