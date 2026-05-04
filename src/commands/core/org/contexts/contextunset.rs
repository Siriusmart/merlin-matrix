use std::error::Error;

use clap::Parser;
use matrix_sdk::async_trait;
use tracing::*;

use crate::{
    commands::{
        Cmd, CmdContext,
        utils::{ErrorMsg, arg_parse, reply_to_html, reply_to_plain},
    },
    org::{Database, contexts::Context, rooms::Room, utils::set_room_context},
};

pub struct CmdContextUnset;

#[derive(Parser)]
#[command(
    name = "ContextUnset",
    version = "0.1.0",
    about = "Unset current room context, this command cannot be disabled"
)]
struct CmdContextUnsetArg;

#[async_trait]
impl Cmd for CmdContextUnset {
    fn permissions(&self) -> &[&str] {
        &[]
    }

    fn default_permission(&self) -> bool {
        true
    }

    #[instrument(skip_all)]
    async fn invoke(&self, context: CmdContext) -> Result<(), Box<dyn Error>> {
        let Some(_) = arg_parse::<CmdContextUnsetArg>(&context).await? else {
            return Ok(());
        };

        let mut conn = Database::conn();

        let room =
            Room::get_or_create(&mut conn, context.room.room_id().strip_sigil().to_string())?;

        let Some(previous_context) = room.context_id() else {
            reply_to_plain(&context, "This room does not have a context set").await?;
            return Ok(());
        };

        let unset_res: Result<(), &'static str> = {
            let power_levels = context.room.power_levels_or_default().await;
            let user_level: i64 = power_levels
                .users
                .get(&context.event.sender)
                .copied()
                .unwrap_or(power_levels.users_default)
                .into();

            if user_level >= 100 {
                set_room_context(&mut conn, room.id(), None)?;
                Ok(())
            } else {
                Err("no permission: require room permission level 100")
            }
        };

        match unset_res {
            Ok(()) => {
                let previous = Context::find(&mut conn, previous_context)?.ok_or(ErrorMsg(
                    "expected context to exist because of foreign key constraint".to_string(),
                ))?;
                reply_to_html(
                    &context,
                    &format!("Room context unset, previously {}", previous.name()),
                    &format!("Room context unset, previously <b>{}</b>", previous.name()),
                )
                .await?;
            }
            Err(err) => {
                reply_to_plain(&context, &format!("Failed to unset room context, {err}")).await?;
            }
        }

        Ok(())
    }
}
