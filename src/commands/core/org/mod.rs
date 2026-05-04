use crate::commands::CmdIndex;

mod contexts;
mod groups;

pub fn register(index: &mut CmdIndex) {
    groups::register(index);
    contexts::register(index);
}
