use std::error::Error;

use clap::Parser;
use matrix_sdk::async_trait;
use tracing::instrument;

use crate::{
    commands::{
        Cmd, CmdContext,
        utils::{arg_parse, reply_to_html, reply_to_plain},
    },
    org::{
        Database,
        context_permissions::{ContextPermission, ContextPermissionPriority},
        contexts::Context,
        groups::Group,
        permissions::Permission,
        users::User,
        utils::{contexts::context_of_room, groups::user_id_in_group_id},
    },
};

pub struct CmdPermSet;

#[derive(Parser)]
#[command(
    name = "PermSet",
    version = "0.1.0",
    about = "Set a permission for a group in a context"
)]
struct CmdPermSetArgs {
    /// Group to set the permission for
    group: String,
    /// Qualifier of permission
    perm: String,
    /// Whether permission is set to allowed or not allowed
    allowed: bool,
    /// Context to set the perm in, default to current room context
    context: Option<String>,
    /// Order at which rules are checked (rule with higher priority overwrites rule with lower priority)
    #[arg(long = "priority", short = 'p')]
    priority: Option<i32>,
}

#[async_trait]
impl Cmd for CmdPermSet {
    fn permissions(&self) -> &[&str] {
        &[
            "core.org.context.perm.set",
            "core.org.context.perm",
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
        let Some(args) = arg_parse::<CmdPermSetArgs>(&context).await? else {
            return Ok(());
        };

        let mut conn = Database::conn();

        let command_sender = User::get_or_create(
            &mut conn,
            context.event.sender.localpart().to_string(),
            context.event.sender.server_name().to_string(),
        )?;

        let context_to_modify = if let Some(context_to_modify) = &args.context {
            let Some(context_to_modify) = Context::find_by_name(&mut conn, context_to_modify)?
            else {
                let name = html_escape::encode_text(context_to_modify);
                reply_to_html(
                    &context,
                    &format!("Could not find a context named {name}"),
                    &format!("Could not find a context named <b>{name}</b>"),
                )
                .await?;
                return Ok(());
            };
            context_to_modify
        } else {
            let Some(context_to_modify) =
                context_of_room(&mut conn, context.room.room_id().strip_sigil())?
            else {
                reply_to_plain(&context, "This room does not have a context set").await?;
                return Ok(());
            };
            context_to_modify
        };

        let sender_is_context_owner = command_sender.id() == context_to_modify.owner();
        let has_perm = if let Some(admin_group_id) = context_to_modify.admin_group() {
            sender_is_context_owner
                || user_id_in_group_id(&mut conn, command_sender.id(), admin_group_id)?
        } else {
            sender_is_context_owner
        };

        if !has_perm {
            reply_to_html(&context, &format!("Permissing denied: you are neither the owner or in the admin group of context {}", context_to_modify.name()), &format!("Permissing denied: you are neither the owner or in the admin group of context <b>{}</b>", context_to_modify.name())).await?;
            return Ok(());
        }

        let Some(group) = Group::find_by_name(&mut conn, &args.group)? else {
            let name = html_escape::encode_text(&args.group);
            reply_to_html(
                &context,
                &format!("Could not find group with name {name}"),
                &format!("Could not find group with name <b>{name}</b>"),
            )
            .await?;
            return Ok(());
        };

        let Some(permission) = Permission::find_by_name(&mut conn, &args.perm)? else {
            let name = html_escape::encode_text(&args.perm);
            reply_to_html(
                &context,
                &format!("Could not find permission with name {name}"),
                &format!("Could not find permission with name <b>{name}</b>"),
            )
            .await?;
            return Ok(());
        };

        ContextPermission::set(
            &mut conn,
            permission.id(),
            group.id(),
            context_to_modify.id(),
            ContextPermissionPriority::new(args.priority.unwrap_or(0)),
            args.allowed,
        )?;

        reply_to_html(
            &context,
            &format!(
                r#"Context Permission Set
* Context - {}
* Permission - {}
* Group - {}
* Allowed - {}"#,
                context_to_modify.name(),
                permission.name(),
                group.name(),
                args.allowed
            ),
            &format!(
                r#"<b>Context Permission Set</b><table>
<tr><td>Context</td><td><b>{}</b></td></tr>
<tr><td>Permission</td><td><b>{}</b></td></tr>
<tr><td>Group</td><td><b>{}</b></td></tr>
<tr><td>Allowed</td><td><b>{}</b></td></tr>
</table>"#,
                context_to_modify.name(),
                permission.name(),
                group.name(),
                args.allowed
            ),
        )
        .await?;

        Ok(())
    }
}
