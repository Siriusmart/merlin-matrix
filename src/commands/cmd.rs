use std::{
    collections::HashMap,
    error::Error,
    ops::Deref,
    sync::{Arc, OnceLock},
};

use matrix_sdk::{
    Client, Room, async_trait, ruma::events::room::message::OriginalSyncRoomMessageEvent,
};

use crate::org::{Database, permissions::Permission};

/// event information passed to the command program
#[derive(Clone)]
pub struct CmdContext(Arc<CmdContextInner>);

impl CmdContext {
    pub fn new(
        client: Client,
        event: OriginalSyncRoomMessageEvent,
        room: Room,
        args: Vec<String>,
    ) -> Self {
        Self(Arc::new(CmdContextInner {
            client,
            event,
            room,
            args,
        }))
    }
}

impl Deref for CmdContext {
    type Target = CmdContextInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct CmdContextInner {
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

pub struct CmdIndex(HashMap<String, Arc<dyn Cmd>>);

impl CmdIndex {
    pub fn register<C: Cmd + 'static>(&mut self, name: &'static str, cmd: C) {
        let mut conn = Database::conn();

        for perm in cmd.permissions() {
            Permission::ensure_exists(&mut conn, perm.to_string())
                .expect("db connection error on boot up");
        }

        if self.0.insert(name.to_string(), Arc::new(cmd)).is_some() {
            panic!("Command clash: name={name}")
        }
    }

    pub fn get(name: &str) -> Option<Arc<dyn Cmd>> {
        CMD_INDEX.get().unwrap().0.get(name).cloned()
    }

    pub fn lock(self) {
        let _ = CMD_INDEX.set(self);
    }

    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self(HashMap::new())
    }
}
