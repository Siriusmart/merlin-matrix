use std::error::Error;

use clap::Parser;
use matrix_sdk::{
    room::{edit::EditedContent, futures::SendMessageLikeEventResult},
    ruma::events::{
        OriginalMessageLikeEvent,
        room::message::{
            AddMentions, ForwardThread, RoomMessageEventContent,
            RoomMessageEventContentWithoutRelation,
        },
    },
};

use crate::commands::CmdContext;

/// parse command arguments, return Some if success, None otherwise
pub async fn arg_parse<P: Parser>(context: &CmdContext) -> Result<Option<P>, Box<dyn Error>> {
    Ok(match P::try_parse_from(&context.args) {
        Ok(p) => Some(p),
        Err(err) => {
            let res = RoomMessageEventContent::text_plain(err.to_string());
            context.room.send(res).await?;
            None
        }
    })
}

/// reply to the message in context
pub async fn reply_to(
    context: &CmdContext,
    content: RoomMessageEventContent,
) -> Result<
    (
        SendMessageLikeEventResult,
        OriginalMessageLikeEvent<RoomMessageEventContent>,
    ),
    Box<dyn Error>,
> {
    let full_event = context
        .event
        .clone()
        .into_full_event(context.room.room_id().to_owned());
    let res = content.make_reply_to(&full_event, ForwardThread::Yes, AddMentions::Yes);
    Ok((context.room.send(res).await?, full_event))
}

