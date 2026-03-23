use std::time::Duration;

use matrix_sdk::{
    Client, Room, event_handler::Ctx, ruma::events::room::member::StrippedRoomMemberEvent,
    sleep::sleep,
};

use crate::config::handlers::OnInviteHandlerConfig;

pub async fn on_invite(
    room_member: StrippedRoomMemberEvent,
    client: Client,
    room: Room,
    config: Ctx<OnInviteHandlerConfig>,
) {
    if room_member.state_key != client.user_id().unwrap() {
        return; // the invite is sent to another user
    }

    tokio::spawn(async move {
        let mut delay = config.initial_delay();
        let mut n = 0;

        while let Err(_err) = room.join().await {
            n += 1;
            delay = config.delay(delay, n);

            if !config.should_retry(delay, n) {
                return;
            }

            sleep(Duration::from_secs_f64(delay)).await;
        }
    });
}
