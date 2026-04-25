use std::error::Error;

use matrix_sdk::{
    room::{edit::EditedContent, futures::SendMessageLikeEventResult},
    ruma::events::room::message::RoomMessageEventContentWithoutRelation,
};

use crate::commands::{CmdContext, utils::reply_to};

pub trait MessageBuffer: Default {
    fn serialise(&self) -> RoomMessageEventContentWithoutRelation;
}

fn push(v: &mut Vec<String>, content: &str, line_break: &str) {
    let mut splitted = content.split(line_break);
    if let Some(last) = v.last_mut() {
        last.push_str(splitted.next().unwrap_or_default());
        v.extend(splitted.map(str::to_string));
    } else {
        *v = splitted.map(str::to_string).collect();
    }
}

fn push_ln(v: &mut Vec<String>, content: &str, line_break: &str) {
    v.extend(content.split(line_break).map(str::to_string));
}

fn replace(v: &mut Vec<String>, content: &str, line_break: &str) {
    *v = content.split(line_break).map(str::to_string).collect();
}

#[derive(Default)]
pub struct PlainMessageBuffer {
    content: Vec<String>,
}

impl PlainMessageBuffer {
    pub fn print(&mut self, content: &str) {
        push(&mut self.content, content, "\n");
    }

    pub fn println(&mut self, content: &str) {
        push_ln(&mut self.content, content, "\n");
    }

    pub fn replace(&mut self, content: &str) {
        replace(&mut self.content, content, "\n");
    }
}

impl MessageBuffer for PlainMessageBuffer {
    fn serialise(&self) -> RoomMessageEventContentWithoutRelation {
        RoomMessageEventContentWithoutRelation::text_plain(self.content.join("\n"))
    }
}

#[derive(Default)]
pub struct HtmlMessageBuffer {
    plain: Vec<String>,
    html: Vec<String>,
}

impl HtmlMessageBuffer {
    pub fn print_plain(&mut self, plain: &str) {
        push(&mut self.plain, plain, "\n");
    }

    pub fn print_html(&mut self, html: &str) {
        push(&mut self.html, html, "<br>");
    }

    pub fn print(&mut self, plain: &str, html: &str) {
        self.print_plain(plain);
        self.print_html(html);
    }

    pub fn println_plain(&mut self, plain: &str) {
        push_ln(&mut self.plain, plain, "\n");
    }

    pub fn println_html(&mut self, html: &str) {
        push_ln(&mut self.html, html, "<br>");
    }

    pub fn println(&mut self, plain: &str, html: &str) {
        self.print_plain(plain);
        self.println_html(html);
    }

    pub fn replace_plain(&mut self, plain: &str) {
        replace(&mut self.plain, plain, "\n");
    }

    pub fn replace_html(&mut self, html: &str) {
        replace(&mut self.html, html, "<br>");
    }

    pub fn replace(&mut self, plain: &str, html: &str) {
        replace(&mut self.plain, plain, "\n");
        replace(&mut self.html, html, "<br>");
    }
}

impl MessageBuffer for HtmlMessageBuffer {
    fn serialise(&self) -> RoomMessageEventContentWithoutRelation {
        RoomMessageEventContentWithoutRelation::text_html(
            self.plain.join("\n"),
            self.html.join("<br>"),
        )
    }
}

/// Printer-like message output
pub struct MessagePrinter<B: MessageBuffer> {
    buffer: B,
    context: CmdContext,
    sent_message: Option<SendMessageLikeEventResult>,
}

impl<B: MessageBuffer> MessagePrinter<B> {
    /// create new printer
    pub fn new_cmd_reply(context: CmdContext) -> Self {
        Self {
            buffer: B::default(),
            context,
            sent_message: None,
        }
    }

    pub fn buffer(&mut self) -> &mut B {
        &mut self.buffer
    }

    /// immediately flush all changes
    pub async fn flush(&mut self) -> Result<(), Box<dyn Error>> {
        if let Some(msg) = &self.sent_message {
            let edited_content = self.buffer.serialise();
            let edited = self
                .context
                .room
                .make_edit_event(
                    &msg.response.event_id,
                    EditedContent::RoomMessage(edited_content),
                )
                .await?;
            self.context.room.send(edited).await?;
        } else {
            self.sent_message = Some(reply_to(&self.context, self.buffer.serialise().into()).await?)
        }

        Ok(())
    }
}
