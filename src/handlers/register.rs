use matrix_sdk::Client;

use crate::handlers::*;

/// these listeners will be registered before first sync
pub fn first_sync(client: &Client) {
    client.add_event_handler(on_invite::on_invite);
}

/// these listeners will be registered after the first sync
pub fn following_syncs(client: &Client) {
    client.add_event_handler(on_maybe_command::on_maybe_command);
}
