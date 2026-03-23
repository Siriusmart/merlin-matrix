use std::{error::Error, fs, ops::Deref, path::Path};

use diesel::{
    SqliteConnection,
    r2d2::{ConnectionManager, Pool},
};

pub mod groups;

type DatabaseBackend = SqliteConnection;
type DatabasePool = Pool<ConnectionManager<DatabaseBackend>>;

/// connection to the sqlite database
#[derive(Clone)]
pub struct Database(DatabasePool);

impl Database {
    /// open a connection
    pub fn open(path: &Path) -> Result<Self, Box<dyn Error>> {
        if !path.parent().unwrap().exists() {
            fs::create_dir_all(path.parent().unwrap())?;
        }

        let manager = ConnectionManager::<DatabaseBackend>::new(path.to_string_lossy());

        Ok(Self(
            Pool::builder()
                .idle_timeout(None) // hopefully disables timeout
                .build(manager)
                .unwrap(),
        ))
    }
}

/// for transprent wrapper struct
impl Deref for Database {
    type Target = DatabasePool;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
