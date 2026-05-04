use std::{collections::HashSet, error::Error};

use clap::Parser;
use matrix_sdk::async_trait;
use tracing::*;

use crate::{
    commands::{
        Cmd, CmdContext,
        utils::{HtmlMessageBuffer, MessagePrinter, arg_parse},
    },
    org::{Database, contexts::Context, users::User},
};

pub struct CmdContextDel;

#[derive(Parser)]
#[command(
    name = "ContextDel",
    version = "0.1.0",
    about = "Delete a context you own"
)]
struct CmdContextDelArg {
    /// List of contexts to delete
    contexts: Vec<String>,
}

#[async_trait]
impl Cmd for CmdContextDel {
    fn permissions(&self) -> &[&str] {
        &[
            "core.org.contexts.delete",
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
        let Some(args) = arg_parse::<CmdContextDelArg>(&context).await? else {
            return Ok(());
        };

        let to_delete = HashSet::<String>::from_iter(args.contexts);

        let mut deleted = HashSet::new();
        let mut not_found = HashSet::new();
        let mut not_owner = HashSet::new();

        let mut conn = Database::conn();

        let command_sender = User::get_or_create(
            &mut conn,
            context.event.sender.localpart().to_string(),
            context.event.sender.server_name().to_string(),
        )?;

        for context in to_delete {
            let Some(context) = Context::find_by_name(&mut conn, &context)? else {
                not_found.insert(context);
                continue;
            };

            if context.owner() != command_sender.id() {
                not_owner.insert(context.name().to_string());
                continue;
            }

            deleted.insert(context.name().to_string());
            context.delete(&mut conn)?;
        }

        let mut msg = MessagePrinter::<HtmlMessageBuffer>::new_cmd_reply(context);

        msg.buffer().print(
            "Context Deletion Summary",
            r#"Context Deletion Summary
<table>"#,
        );

        if deleted.is_empty() {
            msg.buffer().print(
                &format!(
                    "\n* Deleted: {}",
                    deleted.iter().cloned().collect::<Vec<_>>().join(", ")
                ),
                &format!(
                    "<tr><td>Deleted</td><td>{}</td></tr>",
                    deleted
                        .iter()
                        .map(|g| format!("<b>{g}</b>"))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            )
        }

        if not_found.is_empty() {
            msg.buffer().print(
                &format!(
                    "\n* Not deleted (not found): {}",
                    not_found.iter().cloned().collect::<Vec<_>>().join(", ")
                ),
                &format!(
                    "<tr><td>Not deleted (not found)</td><td>{}</td></tr>",
                    not_found
                        .iter()
                        .map(|g| format!("<b>{g}</b>"))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            )
        };

        if not_owner.is_empty() {
            msg.buffer().print(
                &format!(
                    "\n* Not deleted (not owner): {}",
                    not_owner.iter().cloned().collect::<Vec<_>>().join(", ")
                ),
                &format!(
                    "<tr><td>Not deleted (not owner)</td><td>{}</td></tr>",
                    not_owner
                        .iter()
                        .map(|g| format!("<b>{g}</b>"))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            )
        };

        msg.buffer().print_html("</table>");
        msg.flush().await?;

        todo!()
    }
}
