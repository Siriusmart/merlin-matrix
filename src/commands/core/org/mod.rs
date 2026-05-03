use crate::commands::CmdIndex;

mod groups;
mod contexts;

pub fn register(index: &mut CmdIndex) {
    groups::register(index);
}
