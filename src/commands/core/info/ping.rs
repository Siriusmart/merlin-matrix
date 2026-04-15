use std::{error::Error, time::Instant};

use matrix_sdk::{
    async_trait,
    room::edit::EditedContent,
    ruma::events::room::message::{
        AddMentions, ForwardThread, RoomMessageEventContent, RoomMessageEventContentWithoutRelation,
    },
};
use tracing::instrument;

use crate::commands::{Cmd, CmdContext};

pub struct CmdPing;

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
        let full_event = context
            .event
            .into_full_event(context.room.room_id().to_owned());

        let res =
            RoomMessageEventContent::text_html("Latency: measuring", "Latency: <i>measuring</i>")
                .make_reply_to(&full_event, ForwardThread::Yes, AddMentions::Yes);

        let start = Instant::now();
        let sent_msg = context.room.send(res).await?;

        let edited_content =
            EditedContent::RoomMessage(RoomMessageEventContentWithoutRelation::text_html(
                format!("Latency: {}ms", start.elapsed().as_millis()),
                format!("Latency: <b>{}ms</b>", start.elapsed().as_millis()),
            ));
        let edited = context
            .room
            .make_edit_event(&sent_msg.response.event_id, edited_content)
            .await?;
        context.room.send(edited).await?;
        Ok(())
    }
}
