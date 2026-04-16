use std::{error::Error, time::Instant};

use clap::Parser;
use matrix_sdk::async_trait;
use tracing::instrument;

use crate::commands::{Cmd, CmdContext, EditableMessage, utils::arg_parse};

pub struct CmdPing;

#[derive(Parser)]
#[command(name = "Ping", version = "0.1.0", about = "Get bot network latency")]
struct CmdPingArg;

#[async_trait]
impl Cmd for CmdPing {
    fn permissions(&self) -> &[&str] {
        &["core.info.ping", "core.info", "core", "*"]
    }

    fn default_permission(&self) -> bool {
        true
    }

    #[instrument(skip_all)]
    async fn invoke(&self, context: CmdContext) -> Result<(), Box<dyn Error>> {
        if arg_parse::<CmdPingArg>(&context).await?.is_none() {
            return Ok(());
        }

        let start = Instant::now();

        let mut sent_msg =
            EditableMessage::new_reply("Latency: measuring", "Latency: <i>measuring</i>", context)
                .await?;

        let elapsed = start.elapsed().as_millis();

        sent_msg
            .replace(
                &format!("Latency: {elapsed}ms"),
                &format!("Latency: <b>{elapsed}ms</b>"),
            )
            .await?;
        Ok(())
    }
}
