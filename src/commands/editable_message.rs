use std::error::Error;

use matrix_sdk::{
    room::{edit::EditedContent, futures::SendMessageLikeEventResult},
    ruma::events::room::message::{
        RoomMessageEventContent, RoomMessageEventContentWithoutRelation,
    },
};

use crate::commands::{CmdContext, utils::reply_to};

pub struct EditableMessage {
    plain_buffer: Vec<String>,
    html_buffer: Vec<String>,
    sent_message: SendMessageLikeEventResult,
    context: CmdContext,
}

impl EditableMessage {
    /// create a new reply
    pub async fn new_reply(
        plain: &str,
        html: &str,
        context: CmdContext,
    ) -> Result<Self, Box<dyn Error>> {
        let plain_buffer = plain.split("\n").map(str::to_string).collect::<Vec<_>>();
        let html_buffer = html.split("<br>").map(str::to_string).collect::<Vec<_>>();

        let (sent_message, _) = reply_to(
            &context,
            RoomMessageEventContent::text_html(plain_buffer.join("\n"), html_buffer.join("<br>")),
        )
        .await?;

        Ok(Self {
            plain_buffer,
            html_buffer,
            sent_message,
            context,
        })
    }

    /// append to message
    pub async fn print(&mut self, plain: &str, html: &str) -> Result<(), Box<dyn Error>> {
        let mut plain_joined = self.plain_buffer.join("\n");
        plain_joined.push_str(plain);
        let mut html_joined = self.plain_buffer.join("\n");
        html_joined.push_str(html);
        self.replace(&plain_joined, &html_joined).await
    }

    /// append to message on a new line
    pub async fn println(&mut self, plain: &str, html: &str) -> Result<(), Box<dyn Error>> {
        let mut plain_joined = self.plain_buffer.join("\n");
        plain_joined.push('\n');
        plain_joined.push_str(plain);
        let mut html_joined = self.plain_buffer.join("\n");
        html_joined.push_str("<br>");
        html_joined.push_str(html);
        self.replace(&plain_joined, &html_joined).await
    }

    /// replace message content
    pub async fn replace(&mut self, plain: &str, html: &str) -> Result<(), Box<dyn Error>> {
        self.plain_buffer = plain.split("\n").map(str::to_string).collect();
        self.html_buffer = html.split("<br>").map(str::to_string).collect();
        self.flush().await
    }

    async fn flush(&self) -> Result<(), Box<dyn Error>> {
        let edited_content =
            EditedContent::RoomMessage(RoomMessageEventContentWithoutRelation::text_html(
                self.plain_buffer.join("\n"),
                self.html_buffer.join("<br>"),
            ));

        let edited = self
            .context
            .room
            .make_edit_event(&self.sent_message.response.event_id, edited_content)
            .await?;
        self.context.room.send(edited).await?;
        Ok(())
    }
}
