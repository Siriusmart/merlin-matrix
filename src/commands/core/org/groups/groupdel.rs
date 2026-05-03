use std::{collections::HashSet, error::Error};

use clap::Parser;

use matrix_sdk::{async_trait, ruma::events::room::message::RoomMessageEventContent};
use tracing::*;

use crate::{
    commands::{
        Cmd, CmdContext,
        utils::{arg_parse, reply_to},
    },
    org::{Database, groups::Group, users::User},
};

pub struct CmdGroupDel;

#[derive(Parser)]
#[command(
    name = "GroupDel",
    version = "0.1.0",
    about = "Delete an existing group"
)]
struct CmdGroupDelArgs {
    /// List of group names to delete
    groups: Vec<String>,
}

#[async_trait]
impl Cmd for CmdGroupDel {
    fn permissions(&self) -> &[&str] {
        &[
            "core.org.groups.add",
            "core.org.groups",
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
        let Some(args) = arg_parse::<CmdGroupDelArgs>(&context).await? else {
            return Ok(());
        };

        if args.groups.is_empty() {
            reply_to(
                &context,
                RoomMessageEventContent::text_plain("Specify at least one group to delete"),
            )
            .await?;
            return Ok(());
        }

        let to_delete = HashSet::<String>::from_iter(args.groups);

        let mut deleted = HashSet::new();
        let mut not_found = HashSet::new();
        let mut not_owner = HashSet::new();

        let mut conn = Database::conn();

        let command_sender = User::get_or_create(
            &mut conn,
            context.event.sender.localpart().to_string(),
            context.event.sender.server_name().to_string(),
        )?;

        for group in to_delete {
            let group_to_delete = if let Some(g) = Group::find_by_name(&mut conn, &group)? {
                g
            } else {
                not_found.insert(group);
                continue;
            };

            if group_to_delete.owner() != command_sender.id() {
                not_owner.insert(group_to_delete.name().to_string());
                continue;
            }

            deleted.insert(group_to_delete.name().to_string());
            group_to_delete.delete(&mut conn)?;
        }

        let (deleted_plain, deleted_html) = if deleted.is_empty() {
            (String::new(), String::new())
        } else {
            (
                format!(
                    "\n* Deleted: {}",
                    deleted.iter().cloned().collect::<Vec<_>>().join(", ")
                ),
                format!(
                    "<tr><td>Deleted</td><td>{}</td></tr>",
                    deleted
                        .iter()
                        .map(|g| format!("<b>{g}</b>"))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            )
        };

        let (not_found_plain, not_found_html) = if not_found.is_empty() {
            (String::new(), String::new())
        } else {
            (
                format!(
                    "\n* Not deleted (not found): {}",
                    not_found.iter().cloned().collect::<Vec<_>>().join(", ")
                ),
                format!(
                    "<tr><td>Not deleted (not found)</td><td>{}</td></tr>",
                    not_found
                        .iter()
                        .map(|g| format!("<b>{g}</b>"))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            )
        };

        let (not_owner_plain, not_owner_html) = if not_owner.is_empty() {
            (String::new(), String::new())
        } else {
            (
                format!(
                    "\n* Not deleted (not owner): {}",
                    not_owner.iter().cloned().collect::<Vec<_>>().join(", ")
                ),
                format!(
                    "<tr><td>Not deleted (not owner)</td><td>{}</td></tr>",
                    not_owner
                        .iter()
                        .map(|g| format!("<b>{g}</b>"))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            )
        };

        reply_to(
            &context,
            RoomMessageEventContent::text_html(
                format!("Group Deletion Summary{deleted_plain}{not_owner_plain}{not_found_plain}"),
                format!(
                    r#"Group Deletion Summary
<table>{deleted_html}{not_owner_html}{not_found_html}</table>"#
                ),
            ),
        )
        .await?;

        Ok(())
    }
}
