use matrix_sdk::Client;

use crate::commands::cmd::CmdIndex;

pub fn register(client: &Client) {
    let index = CmdIndex::new();
    client.add_event_handler_context(index.lock());
}
