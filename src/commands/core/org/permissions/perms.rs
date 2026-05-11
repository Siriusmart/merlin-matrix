use std::error::Error;

use clap::Parser;
use matrix_sdk::async_trait;
use tracing::instrument;

use crate::{
    commands::{
        Cmd, CmdContext,
        utils::{HtmlMessageBuffer, MessagePrinter, arg_parse, reply_to_html, reply_to_plain},
    },
    org::{
        Database,
        contexts::Context,
        utils::contexts::{ContextPermissionEntry, context_of_room, permissions_of_context},
    },
};

pub struct CmdPerms;

#[derive(Parser)]
#[command(
    name = "Perms",
    version = "0.1.0",
    about = "List all permission rules in a context"
)]
struct CmdPermsArgs {
    /// Name of the context, defaults to current context
    context: Option<String>,
}

#[async_trait]
impl Cmd for CmdPerms {
    fn permissions(&self) -> &[&str] {
        &[
            "core.org.perm.list",
            "core.org.perm",
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
        let Some(args) = arg_parse::<CmdPermsArgs>(&context).await? else {
            return Ok(());
        };

        let mut conn = Database::conn();

        let context_of_interest = if let Some(context_of_interest) = &args.context {
            let Some(context_of_interest) = Context::find_by_name(&mut conn, context_of_interest)?
            else {
                let name = html_escape::encode_text(context_of_interest);
                reply_to_html(
                    &context,
                    &format!("Could not find a context named {name}"),
                    &format!("Could not find a context named <b>{name}</b>"),
                )
                .await?;
                return Ok(());
            };
            context_of_interest
        } else {
            let Some(context_of_interest) =
                context_of_room(&mut conn, context.room.room_id().strip_sigil())?
            else {
                reply_to_plain(&context, "This room does not have a context set").await?;
                return Ok(());
            };
            context_of_interest
        };

        let perm_rules = permissions_of_context(&mut conn, context_of_interest.id())?;

        if perm_rules.is_empty() {
            reply_to_html(
                &context,
                &format!(
                    "Context {} has no permission rules",
                    context_of_interest.name()
                ),
                &format!(
                    "Context <b>{}</b> has no permission rules",
                    context_of_interest.name()
                ),
            )
            .await?;
            return Ok(());
        }

        let mut msg = MessagePrinter::<HtmlMessageBuffer>::new_cmd_reply(context);

        msg.buffer().println(
            &format!("Permission rules of {}", context_of_interest.name()),
            &format!("Permission rules of <b>{}</b>", context_of_interest.name()),
        );

        msg.buffer().print_html(
            "<table><tr><th>Allowed</th><th>Permission</th><th>Group</th><th>Priority</th></tr>",
        );

        for ContextPermissionEntry {
            qualifier,
            allowed,
            group_name,
            priority,
            ..
        } in perm_rules
        {
            let allowed = if allowed { '✓' } else { '✕' };

            msg.buffer().print(
                &format!("\n{allowed} {qualifier} group={group_name} priority={priority}"),
                &format!("<tr><td>{allowed}</td><td>{qualifier}</td><td>{group_name}</td><td>{priority}</td></tr>"),
            );
        }

        msg.buffer().print_html("<table>");
        msg.flush().await?;

        Ok(())
    }
}
