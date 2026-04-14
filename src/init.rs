use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use matrix_sdk::Client;
use tracing::*;

use crate::{
    commands,
    config::{self, ConfigSerde, data::DataConfig},
    org::Database,
};

pub fn init(client: &Client) {
    // add configs to client
    config::register(client);

    // add database connection pool to client
    let data_config = DataConfig::load_write().unwrap();
    let db_pool = Database::open(data_config.sqlite_db_path()).unwrap();
    client.add_event_handler_context(db_pool.clone());

    // run migrations
    const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations/");
    debug!("Running migrations");
    let migrations_count = db_pool
        .get()
        .unwrap()
        .run_pending_migrations(MIGRATIONS)
        .unwrap()
        .len();
    debug!("Successfully ran {migrations_count} migrations");

    // register commands
    commands::register(client);
}
