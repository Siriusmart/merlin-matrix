use matrix_sdk::{
    Client, Room,
    ruma::events::room::message::{OriginalSyncRoomMessageEvent, RoomMessageEventContent},
};

use tracing::*;

use crate::{
    commands::{
        cmd::{CmdContext, CmdIndex},
        utils::reply_to,
    },
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

    for &qualifier in cmd.permissions() {
        let res = match user_has_permission(
            &mut Database::conn(),
            event.sender.localpart(),
            event.sender.server_name().as_str(),
            room.room_id().strip_sigil(),
            qualifier,
        ) {
            Ok(res) => res,
            Err(e) => {
                error!(
                    "Could not fetch permissions user={}:{} room={} ({}) qualifier={qualifier} reason={e}",
                    event.sender.localpart(),
                    event.sender.server_name().as_str(),
                    room.room_id().strip_sigil(),
                    room.name().as_deref().unwrap_or("no name")
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
            "Command denied args={args:?} user={}:{} room={} ({})",
            event.sender.localpart(),
            event.sender.server_name().as_str(),
            room.room_id(),
            room.name().as_deref().unwrap_or("no name")
        );
        return;
    }

    debug!(
        "Command allowed args={args:?} user={}:{} room={} ({})",
        event.sender.localpart(),
        event.sender.server_name().as_str(),
        room.room_id(),
        room.name().as_deref().unwrap_or("no name")
    );

    let context = CmdContext::new(client, event, room.clone(), args.clone());

    // scope so the compiler async check knows res is not held across an await
    // (it ignores drop() for some reason)
    let err_content = {
        let res = cmd.invoke(context.clone()).await;

        if let Err(e) = &res {
            error!(
                "Command error={e:?} args={args:?} user={}:{} room={} ({})",
                context.event.sender.localpart(),
                context.event.sender.server_name().as_str(),
                room.room_id(),
                room.name().as_deref().unwrap_or("no name")
            );

            Some(RoomMessageEventContent::text_html(
                format!("Oops, that's a crash\n{e:?}"),
                format!("Oops, that's a crash<br><pre>{e:?}</pre>"),
            ))
        } else {
            None
        }
    };

    if let Some(err_content) = err_content {
        let _ = reply_to(&context, err_content).await;
    }
}
