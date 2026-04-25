use matrix_sdk::{
    room::futures::SendMessageLikeEventResult,
    ruma::events::room::message::{AddMentions, ForwardThread, RoomMessageEventContent},
};

use crate::commands::CmdContext;

/// reply to the message in context
pub async fn reply_to(
    context: &CmdContext,
    content: RoomMessageEventContent,
) -> Result<SendMessageLikeEventResult, matrix_sdk::Error> {
    let full_event = context
        .event
        .clone()
        .into_full_event(context.room.room_id().to_owned());
    let res = content.make_reply_to(&full_event, ForwardThread::Yes, AddMentions::Yes);
    context.room.send(res).await
}

/// reply to the message in context
pub async fn reply_to_plain(
    context: &CmdContext,
    content: &str,
) -> Result<SendMessageLikeEventResult, matrix_sdk::Error> {
    reply_to(context, RoomMessageEventContent::text_plain(content)).await
}

/// reply to the message in context
pub async fn reply_to_html(
    context: &CmdContext,
    plain: &str,
    html: &str,
) -> Result<SendMessageLikeEventResult, matrix_sdk::Error> {
    reply_to(context, RoomMessageEventContent::text_html(plain, html)).await
}
