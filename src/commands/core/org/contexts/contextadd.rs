use std::error::Error;

use clap::Parser;
use matrix_sdk::async_trait;
use tracing::*;

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
        contexts::{Context, ContextId},
        groups::Group,
        rooms::Room,
        users::User,
        utils::set_room_context,
    },
};

pub struct CmdContextAdd;

#[derive(Parser)]
#[command(name = "ContextAdd", version = "0.1.0", about = "Create a new context")]
struct CmdContextAddArg {
    /// Unique name of the context, e.g. community_name.no_commands
    context: String,
    /// A short description of the group
    #[arg(short = 'd', long = "desc")]
    desc: Option<String>,
    /// @user of group owner
    #[arg(short = 'o', long = "owner")]
    owner: Option<String>,
    #[arg(short = 'a', long = "admin")]
    /// Name of admin group, default: none
    admin_group: Option<String>,
    #[arg(long = "attach")]
    attach: bool,
}

#[async_trait]
impl Cmd for CmdContextAdd {
    fn permissions(&self) -> &[&str] {
        &[
            "core.org.contexts.add",
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
        let Some(args) = arg_parse::<CmdContextAddArg>(&context).await? else {
            return Ok(());
        };

        if !Context::validate_name(&args.context) {
            utils::reply_to_html(
                &context,
                r#"Illegal context name, context name must:
* Be in format "chunks1.chunks2.etc" with at least 2 chunks
* Contain only alphabet/numbers or '-', '_', '.'"#,
                r#"Illegal context name, context name must:
<ul>
<li>Be in format <b>chunks1.chunks2.etc</b> with at least 2 chunks</li>
<li>Contain only alphabet/numbers or '-', '_', '.'</li>
</ul>"#,
            )
            .await?;
            return Ok(());
        }

        // validate description length constraint
        let desc = if let Some(d) = args.desc {
            if Context::desc_max_len() < d.len() {
                utils::reply_to_html(
                    &context,
                    &format!(
                        "Description length must be less than {}, current length: {}",
                        Context::desc_max_len(),
                        d.len()
                    ),
                    &format!(
                        "Description length must be less than <b>{}</b>, current length: <b>{}</b>",
                        Context::desc_max_len(),
                        d.len()
                    ),
                )
                .await?;
                return Ok(());
            } else {
                d
            }
        } else {
            String::new()
        };

        // check group is not a sys.*
        if args.context.starts_with("sys.") {
            utils::reply_to_html(
                &context,
                r#"You are not allowed to make contexts with prefix "sys.""#,
                r#"You are not allowed to make contexts with prefix <code>sys.</code>"#,
            )
            .await?;
            return Ok(());
        }

        let mut conn = Database::conn();

        // check owner exists
        let owner = if let Some(owner) = args.owner {
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
                    user
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
            User::get_or_create(
                &mut conn,
                context.event.sender.localpart().to_string(),
                context.event.sender.server_name().to_string(),
            )?
        };

        let existing_context = Context::find_by_name(&mut conn, &args.context)?;

        // check context name has not been used before
        if let Some(existing_context) = existing_context {
            reply_to_html(
                &context,
                &format!(
                    r#"Not created, there is already another context called "{}""#,
                    existing_context.name()
                ),
                &format!(
                    "Not created, there is already another context called <code>{}</code>",
                    existing_context.name()
                ),
            )
            .await?;
            return Ok(());
        }

        // check admin group exists
        let admin_group = if let Some(admin_group) = args.admin_group {
            let found_group = Group::find_by_name(&mut conn, &admin_group)?;
            if found_group.is_none() {
                reply_to_html(
                    &context,
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

        // actually create the context
        let new_context = Context::create_new(
            &mut conn,
            args.context,
            desc.clone(),
            owner.id(),
            admin_group.as_ref().map(|g| g.id()),
        )?;

        // attach room to new context id if specified
        // - if no attach requested, return none
        // - if attach succeeded, return Some(Ok(previous context))
        // - if attach failed, retrun Some(Err)
        let attach_res: Option<Result<Option<ContextId>, &'static str>> = if args.attach {
            let power_levels = context.room.power_levels_or_default().await;
            let user_level: i64 = power_levels
                .users
                .get(&context.event.sender)
                .copied()
                .unwrap_or(power_levels.users_default)
                .into();

            Some(if user_level >= 100 {
                let room = Room::get_or_create(
                    &mut conn,
                    context.room.room_id().strip_sigil().to_string(),
                )?;

                set_room_context(&mut conn, room.id(), Some(new_context.id()))?;
                Ok(room.context_id())
            } else {
                Err("no permission: require permission level 100")
            })
        } else {
            None
        };

        let mut msg = MessagePrinter::<HtmlMessageBuffer>::new_cmd_reply(context);

        // building summary
        msg.buffer().println(
            &format!(r#"Created Context "{}":"#, new_context.name(),),
            &format!(
                r#"<b>Created Context <code>{}</code></b><table>"#,
                new_context.name()
            ),
        );

        msg.buffer()
            .print("\n* Description - ", "<tr><td>Description</td><td>");

        if desc.is_empty() {
            msg.buffer()
                .print("[empty string]", "<i>[empty string]</i>")
        } else {
            msg.buffer()
                .print(desc.as_str(), &html_escape::encode_text(desc.as_str()))
        }

        msg.buffer().print_html("</td></tr>");

        msg.buffer().print(
            &format!("\n* Owner - {}", owner.display()),
            &format!("<tr><td>Owner</td><td><b>{}</b></td></tr>", owner.display(),),
        );

        if let Some(admin_group) = &admin_group {
            msg.buffer().print(
                &format!("\n* Admin group - {}", admin_group.name()),
                &format!(
                    "<tr><td>Admin group</td><td><b>{}</b></td></tr>",
                    admin_group.name()
                ),
            );
        }

        if let Some(attach_res) = attach_res {
            match attach_res {
                Ok(Some(previous)) => {
                    let previous = Context::find(&mut conn, previous)?.ok_or(ErrorMsg(
                        "expected context to exist because of foreign key constraint".to_string(),
                    ))?;
                    msg.buffer().print(
                        &format!(
                            "\n* Attach to room - succeeded, replaced exsting context {}",
                            previous.name()
                        ),
                        &format!(
                            "<tr><td>Attach to room</td><td>Succeeded, replaced existing context <b>{}</b></td></tr>",
                            previous.name()
                        ),
                    );
                }
                Ok(None) => {
                    msg.buffer().print(
                        "\n* Attach to room - succeeded",
                        "<tr><td>Attach to room</td><td>Succeeded</b></td></tr>",
                    );
                }
                Err(err) => {
                    msg.buffer().print(
                        &format!("\n* Attach to room - failed, err",),
                        &format!("<tr><td>Attach to room</td><td>Failed, {err}</td></tr>",),
                    );
                }
            }
        }

        msg.buffer().print_html("</table>");
        msg.flush().await?;

        todo!()
    }
}
