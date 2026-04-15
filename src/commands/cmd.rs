use std::{collections::HashMap, error::Error, sync::OnceLock};

use matrix_sdk::{
    Client, Room, async_trait, ruma::events::room::message::OriginalSyncRoomMessageEvent,
};

/// event information passed to the command program
pub struct CmdContext {
    pub client: Client,
    pub event: OriginalSyncRoomMessageEvent,
    pub room: Room,
    pub args: Vec<String>,
}

/// A command is a similar to a binary on Unix, it is invokable, and
/// responsible for handling all its arguments, including subcommands
#[async_trait]
pub trait Cmd: Sync + Send {
    /// permissions to be tested whether the command can be ran,
    /// higher is greater priority
    fn permissions(&self) -> &[&str];

    /// if all permissions are a miss, what's the default
    fn default_permission(&self) -> bool;

    /// run the command
    async fn invoke(&self, context: CmdContext) -> Result<(), Box<dyn Error>>;
}

static CMD_INDEX: OnceLock<CmdIndex> = OnceLock::new();

pub struct CmdIndex(HashMap<String, Box<dyn Cmd>>);

impl CmdIndex {
    pub fn register<C: Cmd + 'static>(&mut self, name: &'static str, cmd: C) {
        self.0.insert(name.to_string(), Box::new(cmd));
    }

    pub fn get(name: &str) -> Option<&dyn Cmd> {
        CMD_INDEX.get().unwrap().0.get(name).map(Box::as_ref)
    }

    pub fn lock(self) {
        let _ = CMD_INDEX.set(self);
    }

    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self(HashMap::new())
    }
}
