use std::error::Error;

use clap::Parser;
use matrix_sdk::async_trait;
use tracing::instrument;

use crate::{
    commands::{
        Cmd, CmdContext,
        utils::{
            ErrorMsg, HtmlMessageBuffer, MessagePrinter, arg_parse, reply_to_html, reply_to_plain,
        },
    },
    org::{
        Database,
        contexts::Context,
        groups::Group,
        users::User,
        utils::contexts::{context_of_room, count_rooms_with_context},
    },
};

pub struct CmdContextInfo;

#[derive(Parser)]
#[command(name = "GroupInfo", version = "0.1.0", about = "Show group info")]
struct CmdContextInfoArgs {
    /// Name of the context
    context: Option<String>,
}

#[async_trait]
impl Cmd for CmdContextInfo {
    fn permissions(&self) -> &[&str] {
        &[
            "core.org.contexts.info",
            "core.org.contexts",
            "core.org",
            "core",
            "*",
        ]
    }

    fn default_permission(&self) -> bool {
        true
    }

    #[instrument(skip_all)]
    async fn invoke(&self, context: CmdContext) -> Result<(), Box<dyn Error>> {
        let Some(args) = arg_parse::<CmdContextInfoArgs>(&context).await? else {
            return Ok(());
        };

        let mut conn = Database::conn();

        let room_context = if let Some(name) = &args.context {
            let Some(context) = Context::find_by_name(&mut conn, name)? else {
                reply_to_html(
                    &context,
                    &format!("No contexts with name {name}"),
                    &format!(
                        "No contexts with name<b>{}</b>",
                        html_escape::encode_text(&name)
                    ),
                )
                .await?;
                return Ok(());
            };
            context
        } else {
            let Some(context) = context_of_room(&mut conn, context.room.room_id().strip_sigil())?
            else {
                reply_to_plain(
                    &context,
                    "This room does not have a context set, specify a context to view",
                )
                .await?;
                return Ok(());
            };

            context
        };

        let owner = User::get_with_id(&mut conn, room_context.owner())?.ok_or(ErrorMsg(
            "owner must exist due to foreign key constraint".to_string(),
        ))?;
        let admin_group = if let Some(admin_group_id) = room_context.admin_group() {
            Group::find(&mut conn, admin_group_id)?
        } else {
            None
        };

        let context_room_count = count_rooms_with_context(&mut conn, room_context.id())?;

        let mut msg = MessagePrinter::<HtmlMessageBuffer>::new_cmd_reply(context);

        msg.buffer().println(
            &format!("Context info for \"{}\"", room_context.name()),
            &format!(
                "<b>Context info for <code>{}</code><b>",
                room_context.name()
            ),
        );

        if args.context.is_none() {
            msg.buffer().print(" (active)", " (active)");
        }

        msg.buffer().print_html("<table>");
        if room_context.desc().is_empty() {
            msg.buffer().print(
                "\n* Description - [empty string]",
                "<tr><td>Description</td><td><i>[empty string]</i></td></tr>",
            );
        } else {
            msg.buffer().print(
                &format!("\n* Description - {}", room_context.desc()),
                &format!(
                    "<tr><td>Description</td><td>{}</td></tr>",
                    html_escape::encode_text(room_context.desc())
                ),
            );
        }

        msg.buffer().print(
            &format!("\n* Owner - {}", owner.display()),
            &format!("<tr><td>Owner</td><td><b>{}</b></td></tr>", owner.display()),
        );

        if let Some(admin_group) = admin_group {
            msg.buffer().print(
                &format!("\n* Admin group - {}", admin_group.name()),
                &format!(
                    "<tr><td>Admin group</td><td><b>{}</b></td></tr>",
                    admin_group.name()
                ),
            );
        }

        if context_room_count == 0 {
            msg.buffer().print(
                "\n* Usage - not used in any rooms",
                "<tr><td>Usage</td><td>not used in any rooms</td></tr>",
            );
        } else {
            let s = if context_room_count >= 2 { "s" } else { "" };
            msg.buffer().print(
                &format!("\n* Usage - used in {context_room_count} room{s}",),
                &format!("<tr><td>Usage</td><td>used in {context_room_count} room{s}</td></tr>",),
            );
        }

        msg.buffer().print_html("</table>");
        msg.flush().await?;

        Ok(())
    }
}
