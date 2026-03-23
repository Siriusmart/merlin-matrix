use matrix_sdk::{Client, config::SyncSettings};
use merlin_matrix::{
    config::{self, ConfigSerde, creds::CredsConfig},
    handlers,
};
use tracing::*;
use tracing_subscriber::EnvFilter;

#[instrument(skip_all)]
#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_target(true)
        .with_line_number(true)
        .without_time()
        .init();

    debug!("Reading creds.toml");
    let creds = CredsConfig::load_write().unwrap();

    debug!(
        "Connecting to homeserver={homeserver}",
        homeserver = creds.homeserver()
    );

    let client = Client::builder()
        .homeserver_url(creds.homeserver())
        .build()
        .await
        .expect("failed to create client");

    client
        .matrix_auth()
        .login_username(creds.username(), creds.password())
        .initial_device_display_name("merlin")
        .await
        .expect("matrix auth failed");

    info!(
        "Logged in as {username} on {homeserver}",
        username = creds.username(),
        homeserver = creds.homeserver()
    );

    config::register(&client);
    handlers::register::first_sync(&client);

    debug!("Starting first sync");
    let response = client.sync_once(SyncSettings::default()).await.unwrap();
    debug!("First sync completed");

    handlers::register::following_syncs(&client);

    debug!("Started indefinite syncing");
    client
        .sync(SyncSettings::default().token(response.next_batch))
        .await
        .expect("syncing error");
}
