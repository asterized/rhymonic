mod album;
mod orm;
mod schema;
mod song;

use diesel::SqliteConnection;

pub use album::Album;
pub use orm::Artist;
pub use song::Song;

type Connection = SqliteConnection;

#[derive(Debug)]
pub enum DatabaseError {
    SqlError(diesel::result::Error),
    PoolError(diesel::r2d2::PoolError)
}

impl From<diesel::result::Error> for DatabaseError {
    fn from(value: diesel::result::Error) -> Self {
        DatabaseError::SqlError(value)
    }
}

impl From<diesel::r2d2::PoolError> for DatabaseError {
    fn from(value: diesel::r2d2::PoolError) -> Self {
        DatabaseError::PoolError(value)
    }
}
