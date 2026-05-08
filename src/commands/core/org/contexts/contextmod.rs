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
        Database, contexts::Context, groups::Group, users::User, utils::groups::user_id_in_group_id,
    },
};

pub struct CmdContextMod;

#[derive(Parser)]
#[command(
    name = "ContextMod",
    version = "0.1.0",
    about = "Modify an existing context"
)]
struct CmdContextModArg {
    /// Current name of the context
    context: String,
    /// New name of the context
    #[arg(short = 'n', long = "name")]
    new_name: Option<String>,
    /// New desc of the context
    #[arg(short = 'd', long = "desc")]
    new_desc: Option<String>,
    /// @user of new context owner
    #[arg(short = 'o', long = "owner")]
    new_owner: Option<String>,
    /// Name of the new admin group
    #[arg(short = 'a', long = "admin")]
    new_admin_group: Option<String>,
    /// Remove the existing admin group
    #[arg(long = "remove-admin")]
    remove_admin: bool,
}

#[async_trait]
impl Cmd for CmdContextMod {
    fn permissions(&self) -> &[&str] {
        &[
            "core.org.context.mod",
            "core.org.context",
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
        let Some(args) = arg_parse::<CmdContextModArg>(&context).await? else {
            return Ok(());
        };

        if args.new_name.is_none()
            && args.new_desc.is_none()
            && args.new_owner.is_none()
            && args.new_admin_group.is_none()
            && !args.remove_admin
        {
            reply_to_plain(&context, "Nothing to change: no options provided").await?;
            return Ok(());
        }

        let mut conn = Database::conn();

        let Some(context_to_modify) = Context::find_by_name(&mut conn, &args.context)? else {
            reply_to_html(
                &context,
                &format!(r#"I could not find a context called "{}""#, args.context),
                &format!(
                    "I could not find a context called <code>{}</code>",
                    args.context
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

        // check if command sender have permission to modify context
        let sender_is_context_owner = command_sender.id() == context_to_modify.owner();
        let sender_is_context_admin = if let Some(admin_group_id) = context_to_modify.admin_group()
        {
            sender_is_context_owner
                || user_id_in_group_id(&mut conn, command_sender.id(), admin_group_id)?
        } else {
            sender_is_context_owner
        };

        if !sender_is_context_admin {
            reply_to_html(
                &context,
                &format!(
                    r#"No permission: you are not a context admin of "{}""#,
                    context_to_modify.name()
                ),
                &format!(
                    r#"No permission: you are not a context admin of <code>{}</code>"#,
                    context_to_modify.name()
                ),
            )
            .await?;
            return Ok(());
        }

        let non_owner_offending = if !sender_is_context_owner {
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
                    r#"Cannot change {reason} of "{}"" because you are not the context owner"#,
                    context_to_modify.name()
                ),
                &format!(
                    r#"Cannot change {reason} of <code>{}</code> because you are not the context owner"#,
                    context_to_modify.name()
                ),
            ).await?;
            return Ok(());
        }

        if let Some(name) = &args.new_name {
            if !Context::validate_name(&args.context) {
                reply_to_html(
                    &context,
                    r#"Illegal context name, context name must:
* Be in format chunks1.chunks2.etc with at least 2 chunks
* Contain only alphabet/numbers or '-', '_', '.'"#,
                    r#"Illegal context name, context name must:
<ul>
<li>Be in format chunks1.chunks2.etc with at least 2 chunks</li>
<li>Contain only alphabet/numbers or '-', '_', '.'</li>
</ul>"#,
                )
                .await?;
                return Ok(());
            }

            // check context is not a sys.*
            if name.starts_with("sys.") {
                reply_to_html(
                    &context,
                    r#"You are not allowed to make contexts with prefix "sys.""#,
                    r#"You are not allowed to make contexts with prefix <code>sys.</code>"#,
                )
                .await?;
                return Ok(());
            }
        }

        if let Some(desc) = &args.new_desc
            && Context::desc_max_len() < desc.len()
        {
            reply_to_html(
                &context,
                &format!(
                    "Description length must be less than {}, current length: {}",
                    Context::desc_max_len(),
                    desc.len()
                ),
                &format!(
                    "Description length must be less than <b>{}</b>, current length: <b>{}</b>",
                    Context::desc_max_len(),
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
                            "Not created: new context owner {owner} has never joined a room with Merlin"
                        ), &format!(
                            "Not created: new context owner <b>{owner}</b> has never joined a room with Merlin"
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
            && !name.eq_ignore_ascii_case(context_to_modify.name())
        {
            let existing_context = Context::find_by_name(&mut conn, name)?;

            // check context name has not been used before
            if let Some(existing_context) = existing_context {
                reply_to_html(
                    &context,
                    &format!(
                        r#"Not modified, there is already another context called "{}""#,
                        existing_context.name()
                    ),
                    &format!(
                        "Not modified, there is already another context called <code>{}</code>",
                        existing_context.name()
                    ),
                )
                .await?;
                return Ok(());
            }
        }

        Context::update(
            &mut conn,
            context_to_modify.id(),
            args.new_name.clone(),
            args.new_desc.clone(),
            new_owner.as_ref().map(|u| u.id()),
            new_admin_group,
        )?;

        let mut msg = MessagePrinter::<HtmlMessageBuffer>::new_cmd_reply(context);

        msg.buffer().print(
            &format!(
                "\nContext Update Summary for \"{}\":",
                context_to_modify.name()
            ),
            &format!(
                "<b>Context Update Summary for <code>{}</code></b><table>",
                context_to_modify.name()
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

        msg.buffer().print_html("</table>");
        msg.flush().await?;

        Ok(())
    }
}
