use crate::commands::{
    CmdIndex,
    core::org::contexts::{
        contextadd::CmdContextAdd, contextdel::CmdContextDel, contextinfo::CmdContextInfo,
        contexts::CmdContexts, contextset::CmdContextSet, contextunset::CmdContextUnset,
    },
};

mod contextadd;
mod contextdel;
mod contextinfo;
#[allow(clippy::module_inception)]
mod contexts;
mod contextset;
mod contextunset;

pub fn register(index: &mut CmdIndex) {
    index.register("contextadd", CmdContextAdd);
    index.register("contextdel", CmdContextDel);
    index.register("contextset", CmdContextSet);
    index.register("contextunset", CmdContextUnset);
    index.register("contextinfo", CmdContextInfo);
    index.register("contexts", CmdContexts);
}
