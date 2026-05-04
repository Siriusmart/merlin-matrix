use crate::commands::{
    CmdIndex,
    core::org::contexts::{contextadd::CmdContextAdd, contextset::CmdContextSet},
};

mod contextadd;
mod contextset;

pub fn register(index: &mut CmdIndex) {
    index.register("contextadd", CmdContextAdd);
    index.register("contextset", CmdContextSet);
}
