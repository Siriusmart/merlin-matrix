use matrix_sdk::{
    Client, RoomState,
    event_handler::Ctx,
    room::Room,
    ruma::events::room::message::{MessageType, OriginalSyncRoomMessageEvent},
};
use tracing::*;

use crate::{commands::on_command, config::handlers::OnCommandHandlerConfig};

#[instrument(skip_all)]
pub async fn on_maybe_command(
    event: OriginalSyncRoomMessageEvent,
    client: Client,
    room: Room,
    config: Ctx<OnCommandHandlerConfig>,
) {
    if room.state() != RoomState::Joined {
        return;
    }
    if event.sender == client.user_id().unwrap() {
        return;
    }

    let MessageType::Text(content) = &event.content.msgtype else {
        return;
    };

    if !content.body.starts_with::<&str>(&config.prefix) {
        return;
    }

    let cmd_string = content.body[config.prefix.len()..].to_string();

    on_command(client, event, room, &cmd_string).await;
}
