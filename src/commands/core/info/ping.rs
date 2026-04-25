use std::{error::Error, time::Instant};

use clap::Parser;
use matrix_sdk::async_trait;
use tracing::instrument;

use crate::commands::{
    Cmd, CmdContext,
    utils::{HtmlMessageBuffer, MessagePrinter, arg_parse},
};

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

        let mut printer = MessagePrinter::<HtmlMessageBuffer>::new_cmd_reply(context);
        printer
            .buffer()
            .println("Latency: measuring", "Latency: <i>measuring</i>");

        let start = Instant::now();
        printer.flush().await?;
        let elapsed = start.elapsed().as_millis();

        printer.buffer().replace(
            &format!("Latency: {elapsed}ms"),
            &format!("Latency: <b>{elapsed}ms</b>"),
        );
        printer.flush().await?;

        Ok(())
    }
}
