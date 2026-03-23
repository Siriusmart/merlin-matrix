use matrix_sdk::Client;

use crate::config::{ConfigDe, handlers::HandlersConfig};

pub fn register(client: &Client) {
    // handlers.toml
    let HandlersConfig { on_invite } = HandlersConfig::load().unwrap();
    client.add_event_handler_context(on_invite);
}
