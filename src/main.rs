use matrix_sdk::{Client, config::SyncSettings};
use merlin_matrix::{
    config::{self, ConfigSerde, creds::CredsConfig},
    handlers,
};

#[tokio::main]
async fn main() {
    let creds = CredsConfig::load_write().unwrap();

    let client = Client::builder()
        .homeserver_url(creds.homeserver())
        .build()
        .await
        .expect("failed to create client");

    config::register(&client);

    client
        .matrix_auth()
        .login_username(creds.username(), creds.password())
        .initial_device_display_name("merlin")
        .await
        .expect("matrix auth failed");

    println!("logged in as {}", creds.username());

    handlers::register::first_sync(&client);

    let response = client.sync_once(SyncSettings::default()).await.unwrap();

    handlers::register::following_syncs(&client);

    client
        .sync(SyncSettings::default().token(response.next_batch))
        .await
        .expect("syncing error");
}
