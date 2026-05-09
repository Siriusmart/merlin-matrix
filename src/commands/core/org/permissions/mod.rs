use crate::commands::{
    CmdIndex,
    core::org::permissions::{permset::CmdPermSet, permunset::CmdPermUnset},
};

mod permset;
mod permunset;

pub fn register(index: &mut CmdIndex) {
    index.register("permset", CmdPermSet);
    index.register("permunset", CmdPermUnset);
}
