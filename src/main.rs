use diesel::r2d2::{ConnectionManager, CustomizeConnection, Pool};
use diesel::{RunQueryDsl, SqliteConnection};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};

use iced::futures::channel::mpsc;
use iced::{Subscription, Theme};

use directories::ProjectDirs;

use rfd::FileHandle;

use std::fs::create_dir_all;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;

mod listener;
pub mod model;
mod scan;
mod ui;
pub use model::Album;
pub use model::Song;

use crate::listener::listen;
use crate::ui::Page;

pub fn main() -> iced::Result {
    iced::application(App::new, App::update, App::view)
        .subscription(App::subscription)
        .theme(App::theme)
        .run()
}


#[derive(Debug, Clone)]
pub enum MediaControl {
    Next,
    Prev,
    Resume,
    Pause,
    PlayPause,
}

#[derive(Debug, Clone)]
pub enum Message {
    Queue(Arc<Song>),
    Play(Arc<Song>),
    Media(MediaEvent),
    SetPage(Page),
    SetPosition(f64),
    ScrollPosition(f32),
    Control(MediaControl),

    BeginImportSong,
    BeginImportDir,
    ImportSong(Option<FileHandle>),
    ImportDirectory(Option<FileHandle>),
    DoneImport(Vec<Album>),

    Tick
}

#[derive(Debug, Clone)]
pub enum MediaEvent {
    Connect((mpsc::Sender<MediaSignal>, Arc<AtomicU64>)),
    EndedSong,
    FailedQueue,
    Play,
    Pause,
}

#[derive(Debug, Clone)]
pub enum MediaSignal {
    PlaySong(Arc<Song>),
    NewPosition(u64),
    Pause,
    Play,
}

pub struct App {
    theme: Theme,
    pool: ConnectionPool,
    albums: Vec<Album>,
    songs: Vec<Arc<Song>>,

    queue: Vec<Arc<Song>>,
    queue_position: usize,

    position: Arc<AtomicU64>,

    channel: mpsc::Sender<MediaSignal>,
    connected: bool,
    playing: bool,

    page: Page,
    scroll_position: f32
}

const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

#[derive(Debug)]
struct EnableWal {}

impl CustomizeConnection<SqliteConnection, diesel::r2d2::Error> for EnableWal {
    fn on_acquire(&self, conn: &mut SqliteConnection) -> Result<(), diesel::r2d2::Error> {
        diesel::sql_query("PRAGMA foreign_keys = ON;").execute(conn)?;

        diesel::sql_query("PRAGMA busy_timeout = 5000;").execute(conn)?;

        diesel::sql_query("PRAGMA journal_mode = WAL;").execute(conn)?;

        diesel::sql_query("PRAGMA synchronous = NORMAL;").execute(conn)?;

        Ok(())
    }
}

pub type ConnectionPool = Pool<ConnectionManager<SqliteConnection>>;

fn establish_connection(
    location: impl Into<String>,
) -> Result<ConnectionPool, diesel::r2d2::PoolError> {
    let manager = ConnectionManager::<SqliteConnection>::new(location);

    let pool = Pool::builder()
        .connection_customizer(Box::new(EnableWal {}))
        .build(manager)?;

    pool.get()
        .expect("Could not create connection to database")
        .run_pending_migrations(MIGRATIONS)
        .expect("Could not run migrations");

    Ok(pool)
}

fn establish_in_memory_database() -> ConnectionPool {
    establish_connection(":memory:").unwrap()
}

fn establish_connection_with_fallback(location: &Path) -> ConnectionPool {
    establish_connection(location.to_str().unwrap()).unwrap_or(establish_in_memory_database())
}

const FOLLOWING_SIZE: usize = 50;

impl App {
    fn new() -> Self {
        let location = ProjectDirs::from("io.github", "asterized", "rhymonic");

        let mut pool = if let Some(dirs) = location.as_ref() {
            let data_dir = dirs.data_local_dir();
            let location = data_dir.join("songs.db");

            if !matches!(
                create_dir_all(data_dir).map_err(|x| x.kind()),
                Ok(()) | Err(std::io::ErrorKind::AlreadyExists)
            ) {
                establish_in_memory_database()
            } else {
                let _ = std::fs::File::create_new(&location);
                establish_connection_with_fallback(&location)
            }
        } else {
            establish_in_memory_database()
        };

        let data = Album::load(&mut pool).unwrap();

        let mut songs: Vec<Arc<Song>> = data
            .iter()
            .flat_map(|album| album.iter())
            .map(|song| song.clone())
            .collect();

        songs.sort_unstable();

        let application = Self {
            theme: Theme::Dark,

            pool: pool,

            albums: data,
            songs: songs,

            queue: Vec::new(),
            queue_position: 0,

            position: Arc::new(AtomicU64::new(0)),

            channel: mpsc::channel(0).0,
            connected: false,
            playing: false,

            page: Page::Songs,
            scroll_position: 0f32,
        };

        application
    }

    fn fill_queue(&mut self) -> Option<()> {
        for _ in 0..FOLLOWING_SIZE.saturating_sub(self.queue.len() - self.queue_position) {
            let position = self.songs.binary_search(self.queue.last()?).ok()?;
            self.queue.push(self.songs.get(position + 1)?.clone());
        }

        Some(())
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::run(listen).map(Message::Media)
    }

    fn theme(&self) -> Theme {
        self.theme.clone()
    }
}
