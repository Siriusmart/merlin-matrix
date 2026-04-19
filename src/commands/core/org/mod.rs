use crate::commands::{
    CmdIndex,
    core::org::{
        groupadd::CmdGroupAdd, groupdel::CmdGroupDel, groupmod::CmdGroupMod, groups::CmdGroups,
    },
};

mod groupadd;
mod groupdel;
mod groupmod;
mod groups;

pub fn register(index: &mut CmdIndex) {
    index.register("groupadd", CmdGroupAdd);
    index.register("groupmod", CmdGroupMod);
    index.register("groups", CmdGroups);
    index.register("groupdel", CmdGroupDel);
}
