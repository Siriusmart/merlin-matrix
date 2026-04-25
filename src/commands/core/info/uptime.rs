use std::{error::Error, time::Instant};

use clap::Parser;
use matrix_sdk::async_trait;
use tracing::instrument;

use crate::commands::{
    Cmd, CmdContext,
    utils::{self, arg_parse},
};

pub struct CmdUptime(Instant);

#[derive(Parser)]
#[command(name = "Uptime", version = "0.1.0", about = "Get bot uptime")]
struct CmdUptimeArg;

impl CmdUptime {
    pub fn new() -> Self {
        Self(Instant::now())
    }
}

#[async_trait]
impl Cmd for CmdUptime {
    fn permissions(&self) -> &[&str] {
        &["core.info.uptime", "core.info", "core", "*"]
    }

    fn default_permission(&self) -> bool {
        true
    }

    #[instrument(skip_all)]
    async fn invoke(&self, context: CmdContext) -> Result<(), Box<dyn Error>> {
        if arg_parse::<CmdUptimeArg>(&context).await?.is_none() {
            return Ok(());
        }

        let total_secs = self.0.elapsed().as_secs();
        let units = [
            ("day", total_secs / 60 / 60 / 24),
            ("hour", (total_secs / 60 / 60) % 24),
            ("minute", (total_secs / 60) % 60),
            ("second", total_secs % 60),
        ];

        let chunks = units
            .into_iter()
            .filter(|(_, value)| *value != 0)
            .take(2)
            .collect::<Vec<_>>();

        let (res_body, res_html) = match chunks.as_slice() {
            [] => (
                "Merlin has just started.".to_string(),
                "Merlin has just started.".to_string(),
            ),
            chunks => {
                let time_string = chunks
                    .iter()
                    .map(|(unit, value)| {
                        format!("{value} {unit}{}", if *value > 1 { "s" } else { "" })
                    })
                    .collect::<Vec<_>>()
                    .join(" and ");

                (
                    format!("Merlin has been up for {time_string}."),
                    format!("Merlin has been up for <b>{time_string}</b>"),
                )
            }
        };

        utils::reply_to_html(&context, &res_body, &res_html).await?;

        Ok(())
    }
}
