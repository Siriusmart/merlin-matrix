use crate::commands::{
    CmdIndex,
    core::info::{ping::CmdPing, uptime::CmdUptime},
};

mod ping;
mod uptime;

pub fn register(index: &mut CmdIndex) {
    index.register("ping", CmdPing);
    index.register("uptime", CmdUptime::new());
}
