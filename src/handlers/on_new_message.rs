use matrix_sdk::{
    Client, RoomState,
    room::Room,
    ruma::events::room::message::{
        AddMentions, ForwardThread, MessageType, OriginalSyncRoomMessageEvent,
        RoomMessageEventContent,
    },
};
use tracing::*;

#[instrument(skip_all)]
pub async fn on_new_message(event: OriginalSyncRoomMessageEvent, client: Client, room: Room) {
    if room.state() != RoomState::Joined {
        return;
    }
    if event.sender == client.user_id().unwrap() {
        return;
    }

    let MessageType::Text(content) = &event.content.msgtype else {
        return;
    };

    if content.body == ".ping" {
        let reply = RoomMessageEventContent::text_plain(".ping").make_reply_to(
            &event.into_full_event(room.room_id().to_owned()),
            ForwardThread::Yes,
            AddMentions::Yes,
        );

        if let Err(err) = room.send(reply).await {
            error!("{err}")
        }
    }
}
