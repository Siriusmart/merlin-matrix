use std::error::Error;

use clap::{Parser, error::ErrorKind};
use matrix_sdk::ruma::events::room::message::RoomMessageEventContent;

use crate::commands::{
    CmdContext,
    utils::reply_to::{reply_to, reply_to_html},
};

/// parse command arguments, return Some if success, None otherwise
pub async fn arg_parse<P: Parser>(context: &CmdContext) -> Result<Option<P>, Box<dyn Error>> {
    Ok(match P::try_parse_from(&context.args) {
        Ok(p) => Some(p),
        Err(err)
            if matches!(
                err.kind(),
                ErrorKind::DisplayHelp | ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand
            ) =>
        {
            reply_to_html(context, &err.to_string(), &format!("<pre>{err}</pre>")).await?;
            None
        }
        Err(err) => {
            let res = RoomMessageEventContent::text_plain(err.to_string());
            reply_to(context, res).await?;
            None
        }
    })
}
