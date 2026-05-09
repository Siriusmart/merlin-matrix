use crate::commands::{CmdIndex, core::org::permissions::permset::CmdPermSet};

mod permset;

pub fn register(index: &mut CmdIndex) {
    index.register("permset", CmdPermSet);
}
