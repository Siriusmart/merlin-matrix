use crate::commands::{CmdIndex, core::org::contexts::contextadd::CmdContextAdd};

mod contextadd;

pub fn register(index: &mut CmdIndex) {
    index.register("contextadd", CmdContextAdd);
}
