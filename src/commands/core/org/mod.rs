use crate::commands::{
    CmdIndex,
    core::org::{groupadd::CmdGroupAdd, groupmod::CmdGroupMod},
};

mod groupadd;
mod groupmod;

pub fn register(index: &mut CmdIndex) {
    index.register("groupadd", CmdGroupAdd);
    index.register("groupmod", CmdGroupMod);
}
