use crate::commands::CmdIndex;

mod contexts;
mod groups;
mod permissions;

pub fn register(index: &mut CmdIndex) {
    groups::register(index);
    contexts::register(index);
    permissions::register(index);
}
