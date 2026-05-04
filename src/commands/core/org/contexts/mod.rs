use crate::commands::{
    CmdIndex,
    core::org::contexts::{
        contextadd::CmdContextAdd, contextset::CmdContextSet, contextunset::CmdContextUnset,
    },
};

mod contextadd;
mod contextset;
mod contextunset;

pub fn register(index: &mut CmdIndex) {
    index.register("contextadd", CmdContextAdd);
    index.register("contextset", CmdContextSet);
    index.register("contextunset", CmdContextUnset);
}
