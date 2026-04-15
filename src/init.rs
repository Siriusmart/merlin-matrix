use std::sync::OnceLock;

use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use matrix_sdk::Client;
use tracing::*;

use crate::{
    commands,
    config::{self, ConfigSerde, data::DataConfig},
    org::{Database, DatabaseConnection},
};

static DATABASE: OnceLock<Database> = OnceLock::new();

impl Database {
    /// get a database connection
    pub fn conn() -> DatabaseConnection {
        DATABASE.get().unwrap().get().unwrap()
    }
}

pub fn init(client: &Client) {
    // add configs to client
    config::register(client);

    // add database connection pool to client
    let data_config = DataConfig::load_write().unwrap();
    let db_pool = Database::open(data_config.sqlite_db_path()).unwrap();
    let _ = DATABASE.set(db_pool);

    // run migrations
    const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations/");
    debug!("Running migrations");
    let migrations_count = Database::conn()
        .run_pending_migrations(MIGRATIONS)
        .unwrap()
        .len();
    debug!("Successfully ran {migrations_count} migrations");

    // register commands
    commands::register(client);
}
