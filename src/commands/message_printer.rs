use std::error::Error;

use crate::commands::{CmdContext, EditableMessage};

/// Printer-like message output
#[allow(clippy::large_enum_variant)]
pub enum MessagePrinter {
    Sent(EditableMessage),
    Unsent(CmdContext),
}

impl MessagePrinter {
    /// create new printer
    pub fn new(context: CmdContext) -> Self {
        Self::Unsent(context)
    }

    /// print characters at end
    pub async fn print(&mut self, plain: &str, html: &str) -> Result<(), Box<dyn Error>> {
        match self {
            Self::Sent(m) => m.print(plain, html).await,
            Self::Unsent(context) => {
                *self = Self::Sent(EditableMessage::new_reply(plain, html, context.clone()).await?);
                Ok(())
            }
        }
    }

    /// print characters at new line
    pub async fn println(&mut self, plain: &str, html: &str) -> Result<(), Box<dyn Error>> {
        match self {
            Self::Sent(m) => m.println(plain, html).await,
            Self::Unsent(context) => {
                *self = Self::Sent(EditableMessage::new_reply(plain, html, context.clone()).await?);
                Ok(())
            }
        }
    }

    /// clear message and replace content
    pub async fn replace(&mut self, plain: &str, html: &str) -> Result<(), Box<dyn Error>> {
        match self {
            Self::Sent(m) => m.replace(plain, html).await,
            Self::Unsent(context) => {
                *self = Self::Sent(EditableMessage::new_reply(plain, html, context.clone()).await?);
                Ok(())
            }
        }
    }
}
