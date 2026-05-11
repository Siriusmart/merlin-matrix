use crate::commands::{
    CmdIndex,
    core::org::permissions::{perms::CmdPerms, permset::CmdPermSet, permunset::CmdPermUnset},
};

mod perms;
mod permset;
mod permunset;

pub fn register(index: &mut CmdIndex) {
    index.register("permset", CmdPermSet);
    index.register("permunset", CmdPermUnset);
    index.register("perms", CmdPerms);
}
