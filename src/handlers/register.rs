use matrix_sdk::Client;

use crate::handlers::*;

pub fn first_sync(client: &Client) {
    client.add_event_handler(on_invite::on_invite);
}

pub fn following_syncs(client: &Client) {
    client.add_event_handler(on_new_message::on_new_message);
}
