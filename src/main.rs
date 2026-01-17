use directories::ProjectDirs;
use iced::{Subscription, Theme};
use iced::futures::{channel::mpsc, FutureExt};

use std::thread::available_parallelism;
use std::path::Path;

use diesel::{Connection, SqliteConnection};

pub mod model;
mod scan;
mod schema;
mod ui;
mod listener;
pub use model::Song;

use crate::scan::scan_directory;

pub fn main() -> iced::Result {
    iced::application(App::new, App::update, App::view)
        .subscription(|_| Subscription::run(listener::listen).map(Message::Media))
        .run()
}

#[derive(Debug, Clone)]
pub enum Message {
    Blueify(usize),
    Play(Song),
    Media(MediaEvent)
}

#[derive(Debug, Clone)]
pub enum MediaEvent {
    Playing(usize),
    Connect(mpsc::Sender<MediaSignal>),
    Play,
    Pause
}

pub enum MediaSignal {
    PlaySong(Song),
    AddSong(Song),
    Next
}

pub struct App {
    theme: Theme,
    blue: usize,
    conn: Option<SqliteConnection>,
    songs: Vec<Song>,
    queue: Vec<usize>,
    channel: Option<mpsc::Sender<MediaSignal>>
}

impl App {
    fn new() -> Self {
        let location = ProjectDirs::from("io.github", "asterized", "rhythmic");
        let threads = available_parallelism().map(|x| x.get()).unwrap_or(4);
        let data = scan_directory(Path::new("/share/music/"), threads).unwrap();

        let application = Self {
            theme: Theme::SolarizedDark,
            blue: usize::MAX,
            conn: {
                if location.is_some() {
                    let db_location = location.as_ref().unwrap().data_local_dir().join("songs.db");
                    let conn = SqliteConnection::establish(&db_location.into_os_string().to_string_lossy());
                    conn.ok()
                } else {
                    None
                }
            },
            songs: data,
            queue: Vec::new(),
            channel: None
        };

        application
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::run(listener::listen).map(Message::Media)
    }
}
