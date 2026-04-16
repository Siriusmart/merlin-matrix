mod on_invite;
mod on_maybe_command;
mod on_maybe_new_user;

mod on_ready;
pub use on_ready::on_ready;

use matrix_sdk::Client;

/// these listeners will be registered before first sync
pub fn first_sync(client: &Client) {
    client.add_event_handler(on_invite::on_invite);
}

/// these listeners will be registered after the first sync
pub fn following_syncs(client: &Client) {
    client.add_event_handler(on_maybe_command::on_maybe_command);
    client.add_event_handler(on_maybe_new_user::on_maybe_new_user);
}
