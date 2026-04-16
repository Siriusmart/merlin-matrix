mod cmd;
mod editable_message;
mod entry;
mod message_printer;
mod utils;

pub use cmd::*;
pub use editable_message::EditableMessage;
pub use entry::on_command;
use matrix_sdk::Client;

mod core;

pub fn register(_client: &Client) {
    let mut index = CmdIndex::new();
    core::register(&mut index);
    index.lock();
}
