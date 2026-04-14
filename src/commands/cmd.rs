use std::{collections::HashMap, sync::Arc};

use matrix_sdk::{
    Client, Room, async_trait, ruma::events::room::message::OriginalSyncRoomMessageEvent,
};

pub struct CmdContext {
    pub client: Client,
    pub event: OriginalSyncRoomMessageEvent,
    pub room: Room,
}

#[async_trait]
pub trait Cmd: Sync + Send {
    /// permissions to be tested whether the command can be ran,
    /// higher is greater priority
    fn permissions(&self) -> &[&str];

    /// if all permissions are a miss, what's the default
    fn default_permission(&self) -> bool;

    /// run the command
    async fn invoke(&self, context: CmdContext);
}

#[derive(Clone)]
pub struct CmdIndex(Arc<CmdIndexInner>);
pub struct CmdIndexInner(HashMap<String, Box<dyn Cmd>>);

impl CmdIndexInner {
    pub fn register<C: Cmd + 'static>(&mut self, name: &'static str, cmd: C) {
        self.0.insert(name.to_string(), Box::new(cmd));
    }

    pub fn get(&self, name: &str) -> Option<&dyn Cmd> {
        self.0.get(name).map(Box::as_ref)
    }

    pub fn lock(self) -> CmdIndex {
        CmdIndex(Arc::new(self))
    }
}

impl CmdIndex {
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> CmdIndexInner {
        CmdIndexInner(HashMap::new())
    }
}
