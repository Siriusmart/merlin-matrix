use crate::commands::{
    CmdIndex,
    core::org::groups::{
        groupadd::CmdGroupAdd, groupdel::CmdGroupDel, groupinfo::CmdGroupInfo,
        groupmod::CmdGroupMod, groups::CmdGroups,
    },
};

mod groupadd;
mod groupdel;
mod groupinfo;
mod groupmod;
#[allow(clippy::module_inception)]
mod groups;

pub fn register(index: &mut CmdIndex) {
    index.register("groupadd", CmdGroupAdd);
    index.register("groupmod", CmdGroupMod);
    index.register("groups", CmdGroups);
    index.register("groupdel", CmdGroupDel);
    index.register("groupinfo", CmdGroupInfo);
}
