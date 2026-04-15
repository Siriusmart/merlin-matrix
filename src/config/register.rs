use matrix_sdk::Client;

use crate::config::{ConfigDe, ConfigSerde, data::DataConfig, handlers::HandlersConfig};

/// add client resources
pub fn register(client: &Client) {
    // handlers.toml
    let HandlersConfig {
        on_invite,
        on_command,
    } = HandlersConfig::load().unwrap();
    client.add_event_handler_context(on_invite);
    client.add_event_handler_context(on_command);

    client.add_event_handler_context(DataConfig::load_write().unwrap());
}
