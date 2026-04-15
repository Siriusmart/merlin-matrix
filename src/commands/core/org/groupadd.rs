use std::error::Error;

use clap::Parser;
use matrix_sdk::{async_trait, ruma::events::room::message::RoomMessageEventContent};
use tracing::instrument;

use crate::{
    commands::{
        Cmd, CmdContext, EditableMessage, utils::{self, arg_parse, reply_to}
    },
    org::{Database, groups::Group, users::User},
};

pub struct CmdGroupAdd;

#[derive(Parser)]
#[command(name = "GroupAdd", version = "1.0", about = "Create a new group")]
struct CmdGroupAddArg {
    /// Unique name of the group, e.g. community_name.admins
    group: String,
}

#[async_trait]
impl Cmd for CmdGroupAdd {
    fn permissions(&self) -> &[&str] {
        &["core.org.groupadd", "core.org", "core", "*"]
    }

    fn default_permission(&self) -> bool {
        true
    }

    #[instrument(skip_all)]
    async fn invoke(&self, context: CmdContext) -> Result<(), Box<dyn Error>> {
        let Some(args) = arg_parse::<CmdGroupAddArg>(&context).await? else {
            return Ok(());
        };

        if args.group.starts_with("sys.") {
            utils::reply_to(
                &context,
                RoomMessageEventContent::text_plain(
                    r#"You are not allowed to make groups with prefix "sys.""#,
                ),
            )
            .await?;
            return Ok(());
        }

        let mut conn = Database::conn();

        let user = User::get_or_create(
            &mut conn,
            context.event.sender.localpart().to_string(),
            context.event.sender.server_name().to_string(),
        )?;

        let create_res = Group::create_new(&mut conn, args.group, user.id())?;
        reply_to(&context, RoomMessageEventContent::text_plain("created")).await?;

        Ok(())
    }
}
