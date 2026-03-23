use matrix_sdk::{
    RoomState,
    room::Room,
    ruma::events::room::message::{
        AddMentions, ForwardThread, MessageType, OriginalSyncRoomMessageEvent,
        RoomMessageEventContent,
    },
};

pub async fn on_new_message(event: OriginalSyncRoomMessageEvent, room: Room) {
    if room.state() != RoomState::Joined {
        return;
    }

    let MessageType::Text(content) = &event.content.msgtype else {
        return;
    };

    if content.body == ".ping" {
        let reply = RoomMessageEventContent::text_plain("pong").make_reply_to(
            &event.into_full_event(room.room_id().to_owned()),
            ForwardThread::Yes,
            AddMentions::Yes,
        );

        room.send(reply).await.unwrap();
    }
}
