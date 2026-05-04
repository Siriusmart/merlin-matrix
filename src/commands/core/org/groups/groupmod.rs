use clap::Parser;
use std::{collections::HashSet, error::Error};

use matrix_sdk::async_trait;

use crate::{
    commands::{
        Cmd, CmdContext,
        utils::{
            self, ErrorMsg, HtmlMessageBuffer, MessagePrinter, arg_parse, reply_to_html,
            reply_to_plain,
        },
    },
    org::{
        Database,
        groups::Group,
        users::User,
        utils::{add_user_to_group, remove_user_from_group, user_id_in_group_id},
    },
};

use tracing::*;

pub struct CmdGroupMod;

#[derive(Parser)]
#[command(
    name = "GroupMod",
    version = "0.1.0",
    about = "Modify an existing group"
)]
struct CmdGroupModArg {
    /// Current name of the group
    group: String,
    /// New name of the group
    #[arg(short = 'n', long = "name")]
    new_name: Option<String>,
    /// New desc of the group
    #[arg(short = 'd', long = "desc")]
    new_desc: Option<String>,
    /// @user of new group owner
    #[arg(short = 'o', long = "owner")]
    new_owner: Option<String>,
    /// Name of the new admin group
    #[arg(short = 'a', long = "admin")]
    new_admin_group: Option<String>,
    /// Remove the existing admin group
    #[arg(long = "remove-admin")]
    remove_admin: bool,
    /// A list of @users to add as members of the group, use -u multiple times to add more users
    #[arg(short = 'u', long = "user")]
    add_users: Vec<String>,
    /// A list of @users to remove from members of the group, use -r multiple times to remove more users
    #[arg(short = 'r', long = "remove")]
    remove_users: Vec<String>,
}

#[async_trait]
impl Cmd for CmdGroupMod {
    fn permissions(&self) -> &[&str] {
        &[
            "core.org.groups.mod",
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
        let Some(args) = arg_parse::<CmdGroupModArg>(&context).await? else {
            return Ok(());
        };

        if args.new_name.is_none()
            && args.new_desc.is_none()
            && args.new_owner.is_none()
            && args.new_admin_group.is_none()
            && !args.remove_admin
            && args.add_users.is_empty()
            && args.remove_users.is_empty()
        {
            reply_to_plain(&context, "Nothing to change: no options provided").await?;
            return Ok(());
        }

        let mut conn = Database::conn();

        let Some(group_to_modify) = Group::find_by_name(&mut conn, &args.group)? else {
            reply_to_html(
                &context,
                &format!(r#"I could not find a group called "{}""#, args.group),
                &format!(
                    "I could not find a group called <code>{}</code>",
                    args.group
                ),
            )
            .await?;

            return Ok(());
        };

        let command_sender = User::get_or_create(
            &mut conn,
            context.event.sender.localpart().to_string(),
            context.event.sender.server_name().to_string(),
        )?;

        // check if command sender have permission to modify group
        let current_group_owner = User::get_with_id(&mut conn, group_to_modify.owner())?.ok_or_else(
            || ErrorMsg("groupmod fetch current_group_owner: expected user to exist due to foreign key constraint".to_string())
        )?;

        let sender_is_group_owner = command_sender.id() == current_group_owner.id();
        let sender_is_group_admin = if let Some(admin_group_id) = group_to_modify.admin_group() {
            sender_is_group_owner
                || user_id_in_group_id(&mut conn, command_sender.id(), admin_group_id)?
        } else {
            sender_is_group_owner
        };

        if !sender_is_group_admin {
            reply_to_html(
                &context,
                &format!(
                    r#"No permission: you are not a group admin of "{}""#,
                    group_to_modify.name()
                ),
                &format!(
                    r#"No permission: you are not a group admin of <code>{}</code>"#,
                    group_to_modify.name()
                ),
            )
            .await?;
            return Ok(());
        }

        let non_owner_offending = if !sender_is_group_owner {
            if args.new_admin_group.is_some() {
                Some("admin group")
            } else if args.new_name.is_some() {
                Some("name")
            } else if args.new_owner.is_some() {
                Some("owner")
            } else {
                None
            }
        } else {
            None
        };

        if let Some(reason) = non_owner_offending {
            reply_to_html(&context,
                &format!(
                    r#"Cannot change {reason} of "{}"" because you are not the group owner"#,
                    group_to_modify.name()
                ),
                &format!(
                    r#"Cannot change {reason} of <code>{}</code> because you are not the group owner"#,
                    group_to_modify.name()
                ),
            ).await?;
            return Ok(());
        }

        if let Some(name) = &args.new_name {
            if !Group::validate_name(&args.group) {
                utils::reply_to_html(
                    &context,
                    r#"Illegal group name, group name must:
* Be in format chunks1.chunks2.etc with at least 2 chunks
* Contain only alphabet/numbers or '-', '_', '.'"#,
                    r#"Illegal group name, group name must:
<ul>
<li>Be in format chunks1.chunks2.etc with at least 2 chunks</li>
<li>Contain only alphabet/numbers or '-', '_', '.'</li>
</ul>"#,
                )
                .await?;
                return Ok(());
            }

            // check group is not a sys.*
            if name.starts_with("sys.") {
                utils::reply_to_html(
                    &context,
                    r#"You are not allowed to make groups with prefix "sys.""#,
                    r#"You are not allowed to make groups with prefix <code>sys.</code>"#,
                )
                .await?;
                return Ok(());
            }
        }

        if let Some(desc) = &args.new_desc
            && Group::desc_max_len() < desc.len()
        {
            utils::reply_to_html(
                &context,
                &format!(
                    "Description length must be less than {}, current length: {}",
                    Group::desc_max_len(),
                    desc.len()
                ),
                &format!(
                    "Description length must be less than <b>{}</b>, current length: <b>{}</b>",
                    Group::desc_max_len(),
                    desc.len()
                ),
            )
            .await?;
            return Ok(());
        }

        if args.remove_admin && args.new_admin_group.is_some() {
            reply_to_html(
                &context,
                "Not changed: remove_admin and new_admin_group is both set",
                "Not changed: <i>remove_admin</i> and <i>new_admin_group</i> is both set",
            )
            .await?;
            return Ok(());
        }

        let new_owner = if let Some(owner) = args.new_owner {
            if !owner.starts_with("@") || owner.chars().filter(|c| *c == ':').count() != 1 {
                reply_to_plain(
                    &context,
                    "Owner argument malformed: it should be an @mention",
                )
                .await?;
                return Ok(());
            } else {
                let (m_user_id, m_user_homeserver) = owner[1..].split_once(":").unwrap();
                if let Some(user) = User::get(&mut conn, m_user_id, m_user_homeserver)? {
                    Some(user)
                } else {
                    reply_to_html(
                        &context,
                        &format!(
                            "Not created: new group owner {owner} has never joined a room with Merlin"
                        ), &format!(
                            "Not created: new group owner <b>{owner}</b> has never joined a room with Merlin"
                        ),
                    )
                    .await?;
                    return Ok(());
                }
            }
        } else {
            None
        };

        let (new_admin_group, new_admin_group_name) = if args.remove_admin {
            (Some(None), Some(None))
        } else if let Some(admin_group) = args.new_admin_group {
            if let Some(found_group) = Group::find_by_name(&mut conn, &admin_group)? {
                (
                    Some(Some(found_group.id())),
                    Some(Some(found_group.name().to_string())),
                )
            } else {
                reply_to_html(
                    &context,
                    &format!("Could not find group with name {admin_group}"),
                    &format!("Could not find group with name <i>{admin_group}</i>"),
                )
                .await?;
                return Ok(());
            }
        } else {
            (None, None)
        };

        if let Some(name) = &args.new_name
            && !name.eq_ignore_ascii_case(group_to_modify.name())
        {
            let existing_group = Group::find_by_name(&mut conn, name)?;

            // check group name has not been used before
            if let Some(existing_group) = existing_group {
                reply_to_html(
                    &context,
                    &format!(
                        r#"Not modified, there is already another group called "{}""#,
                        existing_group.name()
                    ),
                    &format!(
                        "Not modified, there is already another group called <code>{}</code>",
                        existing_group.name()
                    ),
                )
                .await?;
                return Ok(());
            }
        }

        Group::update(
            &mut conn,
            group_to_modify.id(),
            args.new_name.clone(),
            args.new_desc.clone(),
            new_owner.as_ref().map(|u| u.id()),
            new_admin_group,
        )?;

        let mut add_users = HashSet::new();
        let mut malformed_add_users = HashSet::new();
        let mut missing_add_users = HashSet::new();

        // put users into 3 buckets
        for user in args.add_users.iter() {
            if user.starts_with("@") && user.chars().filter(|c| *c == ':').count() == 1 {
                let (m_user_id, m_user_homeserver) = user[1..].split_once(":").unwrap();
                match User::get(&mut conn, m_user_id, m_user_homeserver)? {
                    Some(u) => {
                        add_users.insert(u);
                    }
                    None => {
                        missing_add_users.insert(user);
                    }
                }
            } else {
                malformed_add_users.insert(user);
            }
        }

        let mut remove_users = HashSet::new();
        let mut malformed_remove_users = HashSet::new();
        let mut missing_remove_users = HashSet::new();

        // put users into 3 buckets
        for user in args.remove_users.iter() {
            if user.starts_with("@") && user.chars().filter(|c| *c == ':').count() == 1 {
                let (m_user_id, m_user_homeserver) = user[1..].split_once(":").unwrap();
                match User::get(&mut conn, m_user_id, m_user_homeserver)? {
                    Some(u) => {
                        remove_users.insert(u);
                    }
                    None => {
                        missing_remove_users.insert(user);
                    }
                }
            } else {
                malformed_remove_users.insert(user);
            }
        }

        // users actually added to the group
        let mut actually_add_users = HashSet::new();
        // users in add_users that are already in the group
        let mut unchanged_add_users = HashSet::new();

        for user in add_users {
            if add_user_to_group(&mut conn, user.id(), group_to_modify.id())? {
                actually_add_users.insert(user);
            } else {
                unchanged_add_users.insert(user);
            }
        }

        // users actually removed from the group
        let mut actually_remove_users = HashSet::new();
        // users in remove_users that are already not in the group
        let mut unchanged_remove_users = HashSet::new();

        for user in remove_users {
            if remove_user_from_group(&mut conn, user.id(), group_to_modify.id())? {
                actually_remove_users.insert(user);
            } else {
                unchanged_remove_users.insert(user);
            }
        }

        // building summary

        let mut msg = MessagePrinter::<HtmlMessageBuffer>::new_cmd_reply(context);

        msg.buffer().print(
            &format!("\nGroup Update Summary for \"{}\":", group_to_modify.name()),
            &format!(
                "<b>Group Update Summary for <code>{}</code></b><table>",
                group_to_modify.name()
            ),
        );

        if let Some(name) = args.new_name {
            msg.buffer().print(
                &format!("\n* New name: {name}"),
                &format!("<tr><td>New name</td><td><code>{name}</code></td></tr>"),
            )
        }

        if let Some(desc) = args.new_desc {
            msg.buffer()
                .print("\n* New desc:", "<tr><td>New description</td><td>");

            if desc.is_empty() {
                msg.buffer()
                    .print("[empty string]", "<i>[empty string]</i>");
            } else {
                msg.buffer().print(&desc, &html_escape::encode_text(&desc))
            }

            msg.buffer().print_html("</td></tr>");
        }

        if let Some(owner) = new_owner {
            msg.buffer().print(
                &format!("\n* New owner: {}", owner.display()),
                &format!(
                    "<tr><td>New owner</td><td><b>{}</b></td></tr>",
                    owner.display(),
                ),
            )
        }

        match new_admin_group_name {
            Some(Some(admin)) => msg.buffer().print(
                &format!("\n* New admin group: {admin}"),
                &format!("<tr><td>New admin group</td><td><b>{admin}</b></td></tr>"),
            ),
            Some(None) => msg.buffer().print(
                "\n* New admin group: none",
                "<tr><td>New admin group</td><td><i>none</i></td></tr>",
            ),
            _ => {}
        }

        if !actually_add_users.is_empty() {
            let users = actually_add_users.into_iter().collect::<Vec<_>>();
            msg.buffer().print(
                &format!(
                    "\n* Added users - {}",
                    users
                        .iter()
                        .map(|u| u.display().to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
                &format!(
                    "<tr><td>Added users</td><td>{}</td>",
                    users
                        .iter()
                        .map(|u| format!("<b>{}</b>", u.display()))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            )
        };

        if !actually_remove_users.is_empty() {
            let users = actually_remove_users.into_iter().collect::<Vec<_>>();
            msg.buffer().print(
                &format!(
                    "\n* Removed users - {}",
                    users
                        .iter()
                        .map(|u| u.display().to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
                &format!(
                    "<tr><td>Removed users</td><td>{}</td>",
                    users
                        .iter()
                        .map(|u| format!("<b>{}</b>", u.display()))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            )
        };

        if !malformed_add_users.is_empty() {
            let users = malformed_add_users.into_iter().collect::<Vec<_>>();
            msg.buffer().print(
                &format!(
                    "\n* Malformed users (not added) - {}",
                    users
                        .iter()
                        .map(|u| format!("\"{u}\""))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
                &format!(
                    "<tr><td>Malformed users (not added)</td><td>{}</td>",
                    users
                        .iter()
                        .map(|u| format!("<code>{u}</code>"))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            )
        };

        if !malformed_remove_users.is_empty() {
            let users = malformed_remove_users.into_iter().collect::<Vec<_>>();
            msg.buffer().print(
                &format!(
                    "\n* Malformed users (not removed) - {}",
                    users
                        .iter()
                        .map(|u| format!("\"{u}\""))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
                &format!(
                    "<tr><td>Malformed users (not removed)</td><td>{}</td>",
                    users
                        .iter()
                        .map(|u| format!("<code>{u}</code>"))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            )
        };

        if !missing_add_users.is_empty() {
            let users = missing_add_users.into_iter().collect::<Vec<_>>();
            msg.buffer().print(
                &format!(
                    "\n* Missing users (not added) - {}",
                    users
                        .iter()
                        .map(|u| u[1..].to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
                &format!(
                    "<tr><td>Missing users (not added)</td><td>{}</td>",
                    users
                        .iter()
                        .map(|u| format!("<b>{}</b>", &u[1..]))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            )
        };

        if !unchanged_add_users.is_empty() {
            let users = unchanged_add_users.into_iter().collect::<Vec<_>>();
            msg.buffer().print(
                &format!(
                    "\n* Users already in group (not added) - {}",
                    users
                        .iter()
                        .map(|u| u.display().to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
                &format!(
                    "<tr><td>Users already in group (not added)</td><td>{}</td>",
                    users
                        .iter()
                        .map(|u| format!("<b>{}</b>", u.display()))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            )
        };

        if !unchanged_remove_users.is_empty() || !missing_remove_users.is_empty() {
            let users = unchanged_remove_users
                .into_iter()
                .map(|u| u.display().to_string())
                .chain(missing_remove_users.into_iter().map(|s| s[1..].to_string()))
                .collect::<Vec<_>>();
            msg.buffer().print(
                &format!(
                    "\n* Users already not in group (not removed) - {}",
                    users.join(", ")
                ),
                &format!(
                    "<tr><td>Users already not in group (not removed)</td><td>{}</td>",
                    users
                        .iter()
                        .map(|u| format!("<b>{u}</b>"))
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
