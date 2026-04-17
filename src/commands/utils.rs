use std::{error::Error, fmt::Display};

use clap::{Parser, error::ErrorKind};
use matrix_sdk::{
    room::futures::SendMessageLikeEventResult,
    ruma::events::{
        OriginalMessageLikeEvent,
        room::message::{AddMentions, ForwardThread, RoomMessageEventContent},
    },
};

use crate::commands::CmdContext;

#[allow(unused)]
#[derive(Debug)]
pub struct ErrorMsg(pub String);

impl Display for ErrorMsg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{self:?}"))
    }
}

impl Error for ErrorMsg {}

/// parse command arguments, return Some if success, None otherwise
pub async fn arg_parse<P: Parser>(context: &CmdContext) -> Result<Option<P>, Box<dyn Error>> {
    Ok(match P::try_parse_from(&context.args) {
        Ok(p) => Some(p),
        Err(err)
            if matches!(
                err.kind(),
                ErrorKind::DisplayHelp | ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand
            ) =>
        {
            let res =
                RoomMessageEventContent::text_html(err.to_string(), format!("<pre>{err}</pre>"));
            reply_to(context, res).await?;
            None
        }
        Err(err) => {
            let res = RoomMessageEventContent::text_plain(err.to_string());
            reply_to(context, res).await?;
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
    matrix_sdk::Error,
> {
    let full_event = context
        .event
        .clone()
        .into_full_event(context.room.room_id().to_owned());
    let res = content.make_reply_to(&full_event, ForwardThread::Yes, AddMentions::Yes);
    Ok((context.room.send(res).await?, full_event))
}
