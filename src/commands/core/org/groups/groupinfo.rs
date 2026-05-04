use std::error::Error;

use clap::Parser;
use matrix_sdk::async_trait;

use crate::{
    commands::{
        Cmd, CmdContext,
        utils::{ErrorMsg, HtmlMessageBuffer, MessagePrinter, arg_parse, reply_to_html},
    },
    org::{
        Database,
        groups::Group,
        users::User,
        utils::{count_group_members, list_group_members},
    },
};

pub struct CmdGroupInfo;

#[derive(Parser)]
#[command(name = "GroupInfo", version = "0.1.0", about = "Show group info")]
struct CmdGroupInfoArgs {
    group: String,
}

#[async_trait]
impl Cmd for CmdGroupInfo {
    fn permissions(&self) -> &[&str] {
        &[
            "core.org.groups.info",
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
        let Some(args) = arg_parse::<CmdGroupInfoArgs>(&context).await? else {
            return Ok(());
        };

        let mut conn = Database::conn();

        let Some(group) = Group::find_by_name(&mut conn, &args.group)? else {
            reply_to_html(
                &context,
                &format!("Could not find any groups named \"{}\"", args.group),
                &format!("Could not find any groups named <b>{}</b>", args.group),
            )
            .await?;
            return Ok(());
        };

        let owner = User::get_with_id(&mut conn, group.owner())?.ok_or(ErrorMsg(
            "owner must exist due to foreign key constraint".to_string(),
        ))?;
        let admin_group = if let Some(admin_group_id) = group.admin_group() {
            Group::find(&mut conn, admin_group_id)?
        } else {
            None
        };

        let group_members = list_group_members(&mut conn, group.id(), Some(5))?;
        let group_members_count = count_group_members(&mut conn, group.id())?;

        let mut msg = MessagePrinter::<HtmlMessageBuffer>::new_cmd_reply(context);

        msg.buffer().println(
            &format!("Group info for \"{}\"", group.name()),
            &format!("<b>Group info for <code>{}</code><b>", group.name()),
        );

        msg.buffer().print_html("<table>");
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

        if group_members_count == 0 {
            msg.buffer().print(
                "\n* Members - [empty group]",
                "<tr><td>Members</td><td><i>[empty group]</i></td></tr>",
            );
        } else {
            let extras = if group_members_count as usize == group_members.len() {
                String::new()
            } else {
                format!(
                    " and {} more",
                    group_members_count as usize - group_members.len()
                )
            };

            msg.buffer().print(
                &format!(
                    "\n* Members - {}{extras}",
                    group_members
                        .iter()
                        .map(User::display)
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
                &format!(
                    "<tr><td>Members</td><td>{}{extras}</td></tr>",
                    group_members
                        .iter()
                        .map(|u| format!("<b>{}</b>", u.display()))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            );
        }

        msg.buffer().print_html("</table>");
        msg.flush().await?;

        Ok(())
    }
}
