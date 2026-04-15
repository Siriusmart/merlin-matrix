use crate::commands::CmdIndex;

pub mod info;

pub fn register(index: &mut CmdIndex) {
    info::register(index);
}
