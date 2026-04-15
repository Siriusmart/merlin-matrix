use crate::commands::CmdIndex;

pub mod info;
pub mod org;

pub fn register(index: &mut CmdIndex) {
    info::register(index);
    org::register(index);
}
