use std::time::Duration;

use matrix_sdk::{
    Client, Room, RoomMemberships, event_handler::Ctx,
    ruma::events::room::member::StrippedRoomMemberEvent, sleep::sleep,
};
use tracing::*;

use crate::{
    config::handlers::OnInviteHandlerConfig,
    org::{Database, users::User},
};

/// On invited to room, accept it
#[instrument(skip_all)]
pub async fn on_invite(
    room_member: StrippedRoomMemberEvent,
    client: Client,
    room: Room,
    config: Ctx<OnInviteHandlerConfig>,
) {
    info!(
        "user_id={user_id} | room_id={room_id}",
        user_id = room_member.state_key.as_str(),
        room_id = room.room_id(),
    );

    if room_member.state_key != client.user_id().unwrap() {
        debug!("Skipped | reason: user_id mismatch");
        return; // the invite is sent to another user
    }

    tokio::spawn(
        async move {
            let mut delay = config.initial_delay();
            let mut n = 0;

            while let Err(err) = room.join().await {
                n += 1;
                error!("Failed at attempt {n} | reason: {err}");

                delay = config.delay(delay, n);

                if !config.should_retry(delay, n) {
                    error!("Gave up after {n} attempts");
                    return;
                }

                sleep(Duration::from_secs_f64(delay)).await;
            }

            info!("Accepted invite to room_id={}", room.room_id());

            let members = match room.members(RoomMemberships::JOIN).await {
                Ok(m) => m,
                Err(e) => {
                    error!(
                        "Failed to fetch member list for room_id{} reason={e:?}",
                        room.room_id()
                    );
                    return;
                }
            };

            let mut conn = Database::conn();

            for member in members {
                if member.user_id() == client.user_id().unwrap() {
                    continue;
                }

                let res = User::ensure_created(
                    &mut conn,
                    member.user_id().localpart().to_string(),
                    member.user_id().server_name().to_string(),
                );

                if let Err(err) = res {
                    error!(
                        "Could not create user for m_user_id={} reason={err:?}",
                        member.user_id()
                    )
                }
            }
        }
        .in_current_span(),
    );
}
