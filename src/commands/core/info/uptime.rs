use std::{error::Error, time::Instant};

use matrix_sdk::{async_trait, ruma::events::room::message::RoomMessageEventContent};
use tracing::instrument;

use crate::commands::{Cmd, CmdContext};

pub struct CmdUptime(Instant);

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
                    .into_iter()
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

        let res = RoomMessageEventContent::text_html(res_body, res_html);
        context.room.send(res).await?;

        Ok(())
    }
}
