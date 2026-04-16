use std::collections::HashSet;

use matrix_sdk::{Client, RoomMemberships};
use matrix_sdk_base::RoomStateFilter;

use tracing::*;

use crate::org::{Database, users::User};

/// On ready (after first sync), look for all users that shares a room with
/// the bot and add it to the users table
#[instrument(skip_all)]
pub async fn on_ready(client: &Client) {
    let rooms = client.rooms_filtered(RoomStateFilter::JOINED);

    let mut users = HashSet::new();

    for room in rooms {
        let members = match room.members(RoomMemberships::JOIN).await {
            Ok(m) => m,
            Err(err) => {
                error!(
                    "Could not fetch member list room_id={} reason={err:?}",
                    room.room_id()
                );
                continue;
            }
        };

        users.extend(members.into_iter().map(|u| u.user_id().to_owned()));
    }

    let mut conn = Database::conn();

    for user in users {
        if user == client.user_id().unwrap() {
            continue;
        }

        let res = User::ensure_created(
            &mut conn,
            user.localpart().to_string(),
            user.server_name().to_string(),
        );

        if let Err(err) = res {
            error!(
                "Could not create user for m_user_id={} reason={err:?}",
                user
            )
        }
    }
}
