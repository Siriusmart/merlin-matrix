use matrix_sdk::{Client, Room, ruma::events::room::message::OriginalSyncRoomMessageEvent};

use tracing::*;

use crate::{
    commands::cmd::{CmdContext, CmdIndex},
    org::{Database, utils::user_has_permission},
};

pub async fn on_command(
    client: Client,
    event: OriginalSyncRoomMessageEvent,
    room: Room,
    cmd_string: &str,
) {
    let Some(args) = shlex::split(cmd_string) else {
        return;
    };

    if args.is_empty() {
        return;
    }

    let Some(cmd) = CmdIndex::get(&args[0]) else {
        return;
    };

    let mut allowed = cmd.default_permission();

    let Some((room_localpart, room_homeserver)) = room.room_id().as_str()[1..].split_once(":")
    else {
        error!(
            "Could not find room localpart and homeserver from room_id={}",
            room.room_id().as_str()
        );
        return;
    };

    for &qualifier in cmd.permissions() {
        let res = match user_has_permission(
            &mut Database::conn(),
            event.sender.localpart(),
            event.sender.server_name().as_str(),
            room_localpart,
            room_homeserver,
            qualifier,
        ) {
            Ok(res) => res,
            Err(e) => {
                error!(
                    "Could not fetch permissions user={}:{} room={room_localpart}:{room_homeserver} qualifier={qualifier} reason={e}",
                    event.sender.localpart(),
                    event.sender.server_name().as_str()
                );
                return;
            }
        };

        if let Some(perm) = res {
            allowed = perm;
            break;
        }
    }

    if !allowed {
        debug!(
            "Command denied args={args:?} user={}:{} room={room_localpart}:{room_homeserver}",
            event.sender.localpart(),
            event.sender.server_name().as_str()
        );
        return;
    }

    debug!(
        "Command allowed args={args:?} user={}:{} room={room_localpart}:{room_homeserver}",
        event.sender.localpart(),
        event.sender.server_name().as_str()
    );

    let context = CmdContext::new(client, event, room.clone(), args.clone());

    let res = cmd.invoke(context.clone()).await;

    if let Err(e) = res {
        error!(
            "Command {e} args={args:?} user={}:{} room={room_localpart}:{room_homeserver}",
            context.event.sender.localpart(),
            context.event.sender.server_name().as_str()
        )
    }
}
