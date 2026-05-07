use std::error::Error;

use clap::Parser;
use matrix_sdk::async_trait;
use tracing::instrument;

use crate::{
    commands::{
        Cmd, CmdContext,
        utils::{HtmlMessageBuffer, MessagePrinter, arg_parse, reply_to_plain},
    },
    org::{
        Database,
        utils::contexts::{list_user_context_admin_s, list_user_context_owned_s},
    },
};

pub struct CmdContexts;

#[derive(Parser)]
#[command(
    name = "Contexts",
    version = "0.1.0",
    about = "List all contexts the user controls"
)]
struct CmdContextsArgs {
    /// Name of the user
    user: Option<String>,
}

#[async_trait]
impl Cmd for CmdContexts {
    fn permissions(&self) -> &[&str] {
        &[
            "core.org.contexts.list",
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
        let Some(args) = arg_parse::<CmdContextsArgs>(&context).await? else {
            return Ok(());
        };

        let (m_user_id, m_user_homeserver) = if let Some(user) = args.user {
            if !(user.starts_with("@") && user.chars().filter(|c| *c == ':').count() == 1) {
                reply_to_plain(&context, "Malformed user argument, expected @mention.").await?;
                return Ok(());
            }

            let (i, s) = user[1..].split_once(":").unwrap();
            (i.to_string(), s.to_string())
        } else {
            (
                context.event.sender.localpart().to_string(),
                context.event.sender.server_name().to_string(),
            )
        };

        let mut conn = Database::conn();

        let mut msg = MessagePrinter::<HtmlMessageBuffer>::new_cmd_reply(context.clone());

        msg.buffer().println(
            &format!(r#"Context Summary of {m_user_id}:{m_user_homeserver}"#),
            &format!(r#"Context Summary of <b>{m_user_id}:{m_user_homeserver}<b><table>"#),
        );

        let admin_of = list_user_context_admin_s(&mut conn, &m_user_id, &m_user_homeserver)?;
        if !admin_of.is_empty() {
            let s = if admin_of.len() == 1 { "" } else { "s" };
            msg.buffer().print(
                &format!(
                    "\n* Admin of {} context{s}: {}",
                    admin_of.len(),
                    admin_of.join(", ")
                ),
                &format!(
                    "<tr><td>Admin of {} context{s}</td><td><i>{}</i></td></tr>",
                    admin_of.len(),
                    admin_of
                        .iter()
                        .map(|g| format!("<b>{g}</b>"))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            )
        };

        let owner_of = list_user_context_owned_s(&mut conn, &m_user_id, &m_user_homeserver)?;
        if !owner_of.is_empty() {
            let s = if owner_of.len() == 1 { "" } else { "s" };
            msg.buffer().print(
                &format!(
                    "\n* Owner of {} context{s}: {}",
                    owner_of.len(),
                    owner_of.join(", ")
                ),
                &format!(
                    "<tr><td>Owner of {} context{s}</td><td><i>{}</i></td></tr>",
                    owner_of.len(),
                    owner_of
                        .iter()
                        .map(|g| format!("<b>{g}</b>"))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            )
        };

        if owner_of.is_empty() && admin_of.is_empty() {
            reply_to_plain(&context, "This user is not owner or admin of any contexts.").await?;
            return Ok(());
        }

        msg.buffer().print_html("</table>");
        msg.flush().await?;

        Ok(())
    }
}
