use crate::commands::{CmdIndex, core::org::groupadd::CmdGroupAdd};

mod groupadd;

pub fn register(index: &mut CmdIndex) {
    index.register("groupadd", CmdGroupAdd);
}
