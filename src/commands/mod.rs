mod cmd;
mod entry;
mod utils;
mod editable_message;

pub use cmd::*;
pub use entry::on_command;
pub use editable_message::EditableMessage;
use matrix_sdk::Client;

mod core;

pub fn register(_client: &Client) {
    let mut index = CmdIndex::new();
    core::register(&mut index);
    index.lock();
}
