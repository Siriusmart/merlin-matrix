use std::{collections::HashSet, error::Error};

use clap::Parser;
use matrix_sdk::{async_trait, ruma::events::room::message::RoomMessageEventContent};
use tracing::instrument;

use crate::{
    commands::{
        Cmd, CmdContext,
        message_printer::MessagePrinter,
        utils::{self, arg_parse, reply_to},
    },
    org::{Database, groups::Group, users::User, utils::add_user_to_group},
};

pub struct CmdGroupAdd;

#[derive(Parser)]
#[command(name = "GroupAdd", version = "0.1.0", about = "Create a new group")]
struct CmdGroupAddArg {
    /// Unique name of the group, e.g. community_name.admins
    group: String,
    /// A short description of the group
    #[arg(short = 'd', long = "desc")]
    desc: Option<String>,
    /// @user of group owner
    #[arg(short = 'o', long = "owner")]
    owner: Option<String>,
    #[arg(short = 'a', long = "admin")]
    /// Name of admin group, default: none
    admin_group: Option<String>,
    /// A list of @users to add as members of the group, use -u multiple times to add more users
    #[arg(short = 'u', long = "user")]
    users: Vec<String>,
}

#[async_trait]
impl Cmd for CmdGroupAdd {
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
        let Some(args) = arg_parse::<CmdGroupAddArg>(&context).await? else {
            return Ok(());
        };

        // validate group name constraint
        if !Group::validate_name(&args.group) {
            utils::reply_to(
                &context,
                RoomMessageEventContent::text_html(
                    r#"Illegal group name, group name must:
* Be in format "chunks1.chunks2.etc" with at least 2 chunks
* Contain only alphabet/numbers or '-', '_', '.'"#,
                    r#"Illegal group name, group name must:
<ul>
<li>Be in format <b>chunks1.chunks2.etc</b> with at least 2 chunks</li>
<li>Contain only alphabet/numbers or '-', '_', '.'</li>
</ul>"#,
                ),
            )
            .await?;
            return Ok(());
        }

        // validate description length constraint
        let desc = if let Some(d) = args.desc {
            if Group::desc_max_len() < d.len() {
                utils::reply_to(
                    &context,
                    RoomMessageEventContent::text_html(
                        format!("Description length must be less than {}, current length: {}", Group::desc_max_len(), d.len()),
                        format!("Description length must be less than <b>{}</b>, current length: <b>{}</b>", Group::desc_max_len(), d.len()
                    ),
                ))
                .await?;
                return Ok(());
            } else {
                d
            }
        } else {
            String::new()
        };

        // check group is not a sys.*
        if args.group.starts_with("sys.") {
            utils::reply_to(
                &context,
                RoomMessageEventContent::text_html(
                    r#"You are not allowed to make groups with prefix "sys.""#,
                    r#"You are not allowed to make groups with prefix <code>sys.</code>"#,
                ),
            )
            .await?;
            return Ok(());
        }

        let mut conn = Database::conn();

        // check owner exists
        let owner = if let Some(owner) = args.owner {
            if !owner.starts_with("@") || owner.chars().filter(|c| *c == ':').count() != 1 {
                reply_to(
                    &context,
                    RoomMessageEventContent::text_plain(
                        "Owner argument malformed: it should be an @mention",
                    ),
                )
                .await?;
                return Ok(());
            } else {
                let (m_user_id, m_user_homeserver) = owner[1..].split_once(":").unwrap();
                if let Some(user) = User::get(&mut conn, m_user_id, m_user_homeserver)? {
                    user
                } else {
                    reply_to(
                        &context,
                        RoomMessageEventContent::text_html(format!(
                            "Not created: new group owner {owner} has never joined a room with Merlin"
                        ), format!(
                            "Not created: new group owner <b>{owner}</b> has never joined a room with Merlin"
                        )),
                    )
                    .await?;
                    return Ok(());
                }
            }
        } else {
            User::get_or_create(
                &mut conn,
                context.event.sender.localpart().to_string(),
                context.event.sender.server_name().to_string(),
            )?
        };

        let mut users = HashSet::new();
        let mut malformed_users = HashSet::new();
        let mut missing_users = HashSet::new();

        // put users into 3 buckets
        for user in args.users.iter() {
            if user.starts_with("@") && user.chars().filter(|c| *c == ':').count() == 1 {
                let (m_user_id, m_user_homeserver) = user[1..].split_once(":").unwrap();
                match User::get(&mut conn, m_user_id, m_user_homeserver)? {
                    Some(u) => {
                        users.insert(u);
                    }
                    None => {
                        missing_users.insert(user);
                    }
                }
            } else {
                malformed_users.insert(user);
            }
        }

        let mut printer = MessagePrinter::new(context);

        let existing_group = Group::find_by_name(&mut conn, &args.group)?;

        // check group name has not been used before
        if let Some(existing_group) = existing_group {
            printer
                .println(
                    &format!(
                        r#"Not created, there is already another group called "{}""#,
                        existing_group.name()
                    ),
                    &format!(
                        "Not created, there is already another group called <code>{}</code>",
                        existing_group.name()
                    ),
                )
                .await?;
            return Ok(());
        }

        // check admin group exists
        let admin_group = if let Some(admin_group) = args.admin_group {
            let found_group = Group::find_by_name(&mut conn, &admin_group)?;
            if found_group.is_none() {
                printer
                    .println(
                        &format!("Could not find group with name {admin_group}"),
                        &format!("Could not find group with name <i>{admin_group}</i>"),
                    )
                    .await?;
                return Ok(());
            }
            found_group
        } else {
            None
        };

        // actually create the group
        let new_group = Group::create_new(
            &mut conn,
            args.group,
            desc.clone(),
            owner.id(),
            admin_group.map(|g| g.id()),
        )?;

        // add users to group
        for user in users.iter() {
            add_user_to_group(&mut conn, user.id(), new_group.id())?;
        }

        // building summary
        let (users_plain, users_html) = if users.is_empty() {
            (String::new(), String::new())
        } else {
            let users = users.into_iter().collect::<Vec<_>>();
            (
                format!(
                    "\n* Users - {}",
                    users
                        .iter()
                        .map(|u| format!("{}:{}", u.m_id(), u.m_homeserver()))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
                format!(
                    "\n<tr><td>Users</td><td>{}</td>",
                    users
                        .iter()
                        .map(|u| format!("<b>{}:{}</b>", u.m_id(), u.m_homeserver()))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            )
        };

        let (malformed_users_plain, malformed_users_html) = if malformed_users.is_empty() {
            (String::new(), String::new())
        } else {
            let users = malformed_users.into_iter().collect::<Vec<_>>();
            (
                format!(
                    "\n* Malformed users (not added) - {}",
                    users
                        .iter()
                        .map(|u| format!("\"{u}\""))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
                format!(
                    "\n<tr><td>Malformed users (not added)</td><td>{}</td>",
                    users
                        .iter()
                        .map(|u| format!("<code>{u}</code>"))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            )
        };

        let (missing_users_plain, missing_users_html) = if missing_users.is_empty() {
            (String::new(), String::new())
        } else {
            let users = missing_users.into_iter().collect::<Vec<_>>();
            (
                format!(
                    "\n* Missing users (not added) - {}",
                    users
                        .iter()
                        .map(|u| u[1..].to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
                format!(
                    "\n<tr><td>Missing users (not added)</td><td>{}</td>",
                    users
                        .iter()
                        .map(|u| format!("<b>{}</b>", &u[1..]))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            )
        };

        let (description_plain, description_html) = if desc.is_empty() {
            ("[empty string]", "<i>[empty string]</i>".to_string())
        } else {
            (
                desc.as_str(),
                html_escape::encode_text(desc.as_str()).to_string(),
            )
        };

        printer
            .replace(
                &format!(
                    r#"Created Group "{}":
* Description - {description_plain}
* Owner - {}:{}{users_plain}{malformed_users_plain}{missing_users_plain}"#,
                    new_group.name(),
                    owner.m_id(),
                    owner.m_homeserver()
                ),
                &format!(
                    r#"<b>Created Group <code>{}</code></b>
<table>
<tr><td>Description</td><td>{description_html}</td></tr>
<tr><td>Owner</td><td><b>{}:{}</b></td></tr>{users_html}{malformed_users_html}{missing_users_html}
</table>"#,
                    new_group.name(),
                    owner.m_id(),
                    owner.m_homeserver()
                ),
            )
            .await?;

        Ok(())
    }
}
