use diesel::SqliteConnection;
use diesel::connection::SimpleConnection;
use diesel::r2d2::{ConnectionManager, CustomizeConnection, Pool};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};

use iced::futures::channel::mpsc;
use iced::{Subscription, Theme};

use directories::ProjectDirs;

use rfd::FileHandle;

use std::fs::create_dir;
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
    NewPosition(f64),
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
    scroll_position: f32,

    shuffled: bool,
    looped: bool,
}

const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

#[derive(Debug)]
struct EnableWal {}

impl CustomizeConnection<SqliteConnection, diesel::r2d2::Error> for EnableWal {
    fn on_acquire(&self, conn: &mut SqliteConnection) -> Result<(), diesel::r2d2::Error> {
        conn.batch_execute("PRAGMA journal_mode = WAL; PRAGMA synchronous = NORMAL;")?;
        conn.batch_execute("PRAGMA wal_checkpoint(TRUNCATE);")?;
        conn.run_pending_migrations(MIGRATIONS).unwrap();

        Ok(())
    }
}

pub type ConnectionPool = Pool<ConnectionManager<SqliteConnection>>;

fn establish_connection(
    location: impl Into<String>,
) -> Result<ConnectionPool, diesel::r2d2::PoolError> {
    let manager = ConnectionManager::<SqliteConnection>::new(location);

    let pool = Pool::builder()
        .test_on_check_out(true)
        .connection_customizer(Box::new(EnableWal {}))
        .build(manager)?;

    Ok(pool)
}

fn establish_in_memory_database() -> ConnectionPool {
    establish_connection(":memory:").unwrap()
}

fn establish_connection_with_fallback(location: &Path) -> ConnectionPool {
    if !location.exists() {
        create_dir(location).unwrap();
    }

    establish_connection(location.to_str().unwrap()).unwrap_or(establish_in_memory_database())
}

const FOLLOWING_SIZE: usize = 50;

impl App {
    fn new() -> Self {
        let _location = ProjectDirs::from("io.github", "asterized", "rhythmic");

        /*let mut pool = if let Some(dirs) = location.as_ref() {
            let data_dir = dirs.data_local_dir();
            establish_connection_with_fallback(&data_dir.join("songs.db"))
        } else {
            establish_in_memory_database()
        };*/
        let mut pool = establish_in_memory_database();

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

            shuffled: false,
            looped: false,
        };

        application
    }

    fn fill_queue(&mut self) -> Option<()> {
        println!("a");
        for _ in 0..(self.queue.len() + self.queue_position).saturating_sub(FOLLOWING_SIZE) {
            println!("b");
            let position = self.songs.binary_search(self.queue.last()?).ok()?;
            self.queue.push(
                self.songs
                    .get(position + 1)?
                    .clone(),
            );
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
