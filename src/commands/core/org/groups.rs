use std::error::Error;

use clap::Parser;
use matrix_sdk::async_trait;

use crate::{
    commands::{
        Cmd, CmdContext,
        utils::{HtmlMessageBuffer, MessagePrinter, arg_parse, reply_to_plain},
    },
    org::{
        Database,
        utils::{list_user_groups_admin_s, list_user_groups_owned_s, list_user_groups_s},
    },
};

pub struct CmdGroups;

#[derive(Parser)]
#[command(
    name = "Groups",
    version = "0.1.0",
    about = "List all groups the user is in"
)]
struct CmdGroupsArg {
    /// Name of the user
    user: Option<String>,
}

#[async_trait]
impl Cmd for CmdGroups {
    fn permissions(&self) -> &[&str] {
        &[
            "core.org.groups.list",
            "core.org.groups",
            "core.org",
            "core",
            "*",
        ]
    }

    fn default_permission(&self) -> bool {
        true
    }

    async fn invoke(&self, context: CmdContext) -> Result<(), Box<dyn Error>> {
        let Some(args) = arg_parse::<CmdGroupsArg>(&context).await? else {
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

        let mut msg = MessagePrinter::<HtmlMessageBuffer>::new_cmd_reply(context);

        msg.buffer().println(
            &format!(r#"Groups Summary of {m_user_id}:{m_user_homeserver}"#),
            &format!(r#"Groups Summary of <b>{m_user_id}:{m_user_homeserver}<b><table>"#),
        );

        let part_of = list_user_groups_s(&mut conn, &m_user_id, &m_user_homeserver)?;
        if part_of.is_empty() {
            msg.buffer().print(
                "\n* Not member of any groups",
                "<tr><td>Member of 0 groups</td><td><i>[empty list]</i></td></tr>",
            )
        } else {
            let s = if part_of.len() == 1 { "" } else { "s" };
            msg.buffer().print(
                &format!(
                    "\n* Member of {} group{s}: {}",
                    part_of.len(),
                    part_of.join(", ")
                ),
                &format!(
                    "<tr><td>Member of {} group{s}</td><td><i>{}</i></td></tr>",
                    part_of.len(),
                    part_of
                        .iter()
                        .map(|g| format!("<b>{g}</b>"))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            )
        };

        let admin_of = list_user_groups_admin_s(&mut conn, &m_user_id, &m_user_homeserver)?;
        if !admin_of.is_empty() {
            let s = if admin_of.len() == 1 { "" } else { "s" };
            msg.buffer().print(
                &format!(
                    "\n* Admin of {} group{s}: {}",
                    admin_of.len(),
                    admin_of.join(", ")
                ),
                &format!(
                    "<tr><td>Admin of {} group{s}</td><td><i>{}</i></td></tr>",
                    admin_of.len(),
                    admin_of
                        .iter()
                        .map(|g| format!("<b>{g}</b>"))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            )
        };

        let owner_of = list_user_groups_owned_s(&mut conn, &m_user_id, &m_user_homeserver)?;
        if !owner_of.is_empty() {
            let s = if owner_of.len() == 1 { "" } else { "s" };
            msg.buffer().print(
                &format!(
                    "\n* Owner of {} group{s}: {}",
                    owner_of.len(),
                    owner_of.join(", ")
                ),
                &format!(
                    "<tr><td>Owner of {} group{s}</td><td><i>{}</i></td></tr>",
                    owner_of.len(),
                    owner_of
                        .iter()
                        .map(|g| format!("<b>{g}</b>"))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            )
        };

        msg.buffer().print_html("</table>");
        msg.flush().await?;

        Ok(())
    }
}
