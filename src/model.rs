mod album;
mod orm;
mod schema;
mod song;

use diesel::SqliteConnection;

pub use album::Album;
pub use orm::Artist;
pub use orm::DbAlbum;
pub use song::{Song, UncommittedSong};

type Connection = SqliteConnection;

trait Hashed {
    fn get_hash(&self) -> &[u8];
}

macro_rules! autohashed {
    ( $( $x:ty ),* ) => {
        $(
            impl Hashed for $x {
                fn get_hash(&self) -> &[u8] {
                    &self.hash
                }
            }

            impl Hashed for std::sync::Arc<$x> {
                fn get_hash(&self) -> &[u8] {
                    &self.hash
                }
            }
        )*
    }
}

autohashed!(Song, UncommittedSong, Album);

fn combine<'a, const T: usize>(a: &'a [u8; T], b: &'a [u8; T]) -> [u8; T] {
    a.iter()
        .zip(b)
        .map(|(i, j)| i ^ j)
        .collect::<Vec<u8>>()
        .try_into()
        .unwrap()
}

#[macro_export]
macro_rules! warn {
    ($message:expr) => {
        eprintln!("WARN: {}", $message)
    };
}

#[derive(Debug)]
pub enum DatabaseError {
    SqlError(diesel::result::Error),
    PoolError(diesel::r2d2::PoolError),
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
