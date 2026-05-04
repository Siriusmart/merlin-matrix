use crate::commands::{
    CmdIndex,
    core::org::contexts::{
        contextadd::CmdContextAdd, contextdel::CmdContextDel, contextset::CmdContextSet,
        contextunset::CmdContextUnset,
    },
};

mod contextadd;
mod contextdel;
mod contextset;
mod contextunset;

pub fn register(index: &mut CmdIndex) {
    index.register("contextadd", CmdContextAdd);
    index.register("contextdel", CmdContextDel);
    index.register("contextset", CmdContextSet);
    index.register("contextunset", CmdContextUnset);
}
