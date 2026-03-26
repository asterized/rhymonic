use diesel::connection::SimpleConnection;
use diesel::r2d2::{ConnectionManager, CustomizeConnection, Pool};
use directories::ProjectDirs;
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use iced::futures::channel::mpsc;
use iced::{Subscription, Theme};

use std::path::Path;
use std::sync::Arc;
use std::fs::create_dir;
use std::sync::atomic::{AtomicU64, AtomicUsize};
use std::time::Duration;

use diesel::SqliteConnection;

pub mod ellipsize;
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
pub enum Message {
    Queue(Arc<Song>),
    Play(Arc<Song>),
    Media(MediaEvent),
    Send(MediaSignal),
    SetPage(Page),
    QueueStep(isize),
    SetPosition(f64),
    PlayPause,
    None
}

#[derive(Debug, Clone)]
pub enum MediaEvent {
    Connect(mpsc::Sender<MediaSignal>),
    Sync(Duration),
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
    InitialPosition(Arc<AtomicU64>),
    NewPosition(f64),
    Sync,
    PlayPause,
    Next,
}

pub struct App {
    theme: Theme,
    pool: Option<ConnectionPool>,
    albums: Arc<Vec<Album>>,
    songs: Vec<Arc<Song>>,

    queue: Vec<Arc<Song>>,
    queue_position: usize,

    position: f64,

    channel: mpsc::Sender<MediaSignal>,
    connected: bool,
    playing: bool,

    page: Page,

    shuffled: bool,
    looped: bool
}

const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

#[derive(Debug)]
struct EnableWal {}

impl CustomizeConnection<SqliteConnection, diesel::r2d2::Error> for EnableWal {
    fn on_acquire(&self, conn: &mut SqliteConnection) -> Result<(), diesel::r2d2::Error> {
        conn.batch_execute("PRAGMA journal_mode = WAL; PRAGMA synchronous = NORMAL;")?;

        Ok(())
    }
}

pub type ConnectionPool = Pool<ConnectionManager<SqliteConnection>>;

fn establish_connection(directory: &Path) -> Result<ConnectionPool, diesel::r2d2::PoolError> {
    let _ = create_dir(directory);

    let db_location = directory.join("songs.db");

    let manager = ConnectionManager::<SqliteConnection>::new(db_location.to_str().unwrap());

    let pool = Pool::builder()
        .test_on_check_out(true)
        .connection_customizer(Box::new(EnableWal {}))
        .build(manager)?;

    let _ = pool.get()?.run_pending_migrations(MIGRATIONS);

    Ok(pool)
}

impl App {
    fn new() -> Self {
        let location = ProjectDirs::from("io.github", "asterized", "rhythmic");

        let mut pool = if let Some(dirs) = location.as_ref() {
            let data_dir = dirs.data_local_dir();
            establish_connection(data_dir).ok()
        } else {
            None
        };

        /*
        let data = scan_directory(Path::new("/share/music/")).unwrap();

        if let Some(ref mut connection) = conn {
            for album in &data {
                let _ = album.commit(connection);
            }
        }
        */

        let data = Album::load(pool.as_mut().unwrap()).unwrap();

        let mut songs: Vec<Arc<Song>> = data.iter().flat_map(|album| album.iter()).map(|song| song.clone()).collect();
        songs.sort_unstable();

        let application = Self {
            theme: Theme::Dark,

            pool: pool,

            albums: Arc::new(data),
            songs: songs,

            queue: Vec::new(),
            queue_position: 0,

            position: 0f64,

            channel: mpsc::channel(0).0,
            connected: false,
            playing: false,

            page: Page::Songs,
            shuffled: false,
            looped: false
        };

        application
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch(
            [
                Subscription::run(listen).map(Message::Media),
                iced::time::every(Duration::from_millis(100))
                    .map(|_| Message::Send(MediaSignal::Sync))
            ]
        )
    }

    fn theme(&self) -> Theme {
        self.theme.clone()
    }
}
