mod album;
mod orm;
mod schema;
mod song;
pub use album::Album;
use diesel::SqliteConnection;
pub use song::Song;

type Connection = SqliteConnection;
