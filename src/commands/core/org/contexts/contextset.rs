use std::error::Error;

use clap::Parser;
use matrix_sdk::async_trait;

use crate::{
    commands::{
        Cmd, CmdContext,
        utils::{ErrorMsg, arg_parse, reply_to_html, reply_to_plain},
    },
    org::{
        Database,
        contexts::{Context, ContextId},
        rooms::Room,
        utils::set_room_context,
    },
};
use tracing::*;

pub struct CmdContextSet;

#[derive(Parser)]
#[command(
    name = "ContextSet",
    version = "0.1.0",
    about = "Set current room to a context, this command cannot be disabled"
)]
struct CmdContextSetArg {
    /// Unique name of the context, e.g. community_name.no_commands
    context: String,
}

#[async_trait]
impl Cmd for CmdContextSet {
    fn permissions(&self) -> &[&str] {
        &[]
    }

    fn default_permission(&self) -> bool {
        true
    }

    #[instrument(skip_all)]
    async fn invoke(&self, context: CmdContext) -> Result<(), Box<dyn Error>> {
        let Some(args) = arg_parse::<CmdContextSetArg>(&context).await? else {
            return Ok(());
        };

        let mut conn = Database::conn();
        let Some(requested_context) = Context::find_by_name(&mut conn, &args.context)? else {
            reply_to_html(
                &context,
                &format!("No context found with name {}", args.context),
                &format!(
                    "No context found with name <code>{}</code>",
                    html_escape::encode_text(&args.context)
                ),
            )
            .await?;
            return Ok(());
        };

        let attach_res: Result<Option<ContextId>, &'static str> = {
            let power_levels = context.room.power_levels_or_default().await;
            let user_level: i64 = power_levels
                .users
                .get(&context.event.sender)
                .copied()
                .unwrap_or(power_levels.users_default)
                .into();

            if user_level >= 100 {
                let room = Room::get_or_create(
                    &mut conn,
                    context.room.room_id().strip_sigil().to_string(),
                )?;

                set_room_context(&mut conn, room.id(), Some(requested_context.id()))?;
                Ok(room.context_id())
            } else {
                Err("no permission: require room permission level 100")
            }
        };

        match attach_res {
            Ok(Some(previous)) => {
                let previous = Context::find(&mut conn, previous)?.ok_or(ErrorMsg(
                    "expected context to exist because of foreign key constraint".to_string(),
                ))?;
                reply_to_html(
                    &context,
                    &format!(
                        "Room context set, replacing existing context {}",
                        previous.name()
                    ),
                    &format!(
                        "Room context set, replacing existing context <b>{}</b>",
                        previous.name()
                    ),
                )
                .await?;
            }
            Ok(None) => {
                reply_to_plain(&context, "Room context set").await?;
            }
            Err(err) => {
                reply_to_plain(&context, &format!("Failed to set room context, {err}")).await?;
            }
        }

        Ok(())
    }
}
