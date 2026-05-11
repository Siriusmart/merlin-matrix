use std::{collections::HashMap, error::Error, fmt::Write};

use clap::{Parser, error::ErrorKind};
use matrix_sdk::ruma::events::room::message::RoomMessageEventContent;

use crate::commands::{
    CmdContext, CmdIndex,
    utils::{
        reply_to::{reply_to, reply_to_html},
        reply_to_plain,
    },
};

/// parse command arguments, return Some if success, None otherwise (to abort command)
pub async fn arg_parse<P: Parser>(context: &CmdContext) -> Result<Option<P>, Box<dyn Error>> {
    if context.args.iter().any(|s| s == "-P" || s == "--perms") {
        let name = context.args.first().map(String::as_str).unwrap_or("");
        let Some(cmd) = CmdIndex::get(name) else {
            reply_to_plain(context, "Command permissions not found").await?;
            return Ok(None);
        };

        let allowed = if cmd.default_permission() {
            "Allowed"
        } else {
            "Disallowed"
        };

        reply_to_html(
            context,
            &format!(
                r#"Permissions for .{name}
Default: {allowed}

Overriding permissions: {}"#,
                if cmd.permissions().is_empty() {
                    "none".to_string()
                } else {
                    cmd.permissions()
                        .iter()
                        .map(|s| format!("\n* {s}"))
                        .collect::<String>()
                }
            ),
            &format!(
                r#"<h1>Permissions for .{name}</h1>
Default: {allowed}

<h4>Overriding permissions</h4>
{}"#,
                if cmd.permissions().is_empty() {
                    "No overriding permissions".to_string()
                } else {
                    format!(
                        "<ul>{}</ul>",
                        cmd.permissions()
                            .iter()
                            .map(|s| format!("<li>{s}</li>"))
                            .collect::<String>()
                    )
                },
            ),
        )
        .await?;
        return Ok(None);
    }

    Ok(match P::try_parse_from(&context.args) {
        Ok(p) => Some(p),
        Err(err)
            if matches!(
                err.kind(),
                ErrorKind::DisplayHelp | ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand
            ) =>
        {
            let mut help = HelpMessage::parse(
                context
                    .args
                    .first()
                    .map(String::as_str)
                    .unwrap_or("")
                    .to_string(),
                &err.to_string(),
            );

            help.options.push((
                "-P, --perms".to_string(),
                "Print command permissions".to_string(),
            ));

            reply_to_html(context, &help.plain(), &help.html()).await?;
            None
        }
        Err(err) => {
            let res = RoomMessageEventContent::text_plain(err.to_string());
            reply_to(context, res).await?;
            None
        }
    })
}

/// parsed help message
struct HelpMessage {
    name: String,
    description: String,
    usage: String,
    /// tag (if None is positional) and description
    args: Vec<(Option<String>, String)>,
    /// flags and description
    options: Vec<(String, String)>,
}

impl HelpMessage {
    pub fn parse(name: String, s: &str) -> Self {
        let mut lines = s.lines();
        let description = lines.next().unwrap_or_default().to_string();
        lines.next();
        let usage = lines
            .next()
            .unwrap_or_default()
            .split_whitespace()
            .filter(|s| !s.is_empty() || *s == "Usage:")
            .skip(1)
            .collect::<Vec<_>>()
            .join(" ");

        let mut sections_map: HashMap<String, Vec<String>> = HashMap::new();
        let mut section: Option<String> = None;

        for line in lines {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            if line.ends_with(":")
                && line[0..line.len() - 1]
                    .chars()
                    .all(|c| c.is_ascii_alphabetic())
            {
                section = Some(line[0..line.len() - 1].to_string());
                continue;
            }

            let Some(section) = &section else { continue };
            sections_map
                .entry(section.clone())
                .or_default()
                .push(line.to_string());
        }

        let args = sections_map
            .remove("Arguments")
            .unwrap_or_default()
            .iter()
            .map(String::as_str)
            .map(str::trim)
            .map(|s| {
                if s.starts_with("[") {
                    let (tag, content) = s.split_once(" ").unwrap_or((s, ""));
                    (Some(tag.trim().to_string()), content.trim().to_string())
                } else {
                    (None, s.to_string())
                }
            })
            .collect();

        let options = sections_map
            .remove("Options")
            .unwrap_or_default()
            .iter()
            .map(String::as_str)
            .map(str::trim)
            .map(|s| {
                let mut flags = Vec::new();
                let mut desc = String::new();

                let mut words = s.split(" ");
                for word in words.by_ref() {
                    if word.starts_with("-") {
                        flags.push(word);
                    } else {
                        desc.push_str(word);
                        break;
                    }
                }

                for word in words {
                    desc.push(' ');
                    desc.push_str(word);
                }

                (flags.join(" "), desc)
            })
            .collect();

        Self {
            name,
            description,
            usage,
            args,
            options,
        }
    }

    pub fn plain(&self) -> String {
        let mut out = format!("Help Page for .{}", self.name);
        writeln!(&mut out, "{}", self.description).expect("how could it even fail");
        writeln!(&mut out, "\nUsage: {}", self.usage).expect("how could it even fail");

        if !self.args.is_empty() {
            writeln!(&mut out, "\nArguments:").expect("how could it even fail");

            let longest_tag = self
                .args
                .iter()
                .map(|arg| arg.0.as_deref().unwrap_or("(positional)").len())
                .max()
                .unwrap_or(0);

            for (tag, desc) in self.args.iter() {
                let tag = tag.as_deref().unwrap_or("(positional)");
                let pad = " ".repeat(longest_tag - tag.len());

                writeln!(&mut out, "  {tag}{pad}  {desc}").expect("how could it even fail");
            }
        }

        if !self.options.is_empty() {
            writeln!(&mut out, "\nOptions:").expect("how could it even fail");

            let longest_tag = self
                .options
                .iter()
                .map(|arg| arg.0.len())
                .max()
                .unwrap_or(0);

            for (tag, desc) in self.options.iter() {
                let pad = " ".repeat(longest_tag - tag.len());

                writeln!(&mut out, "  {tag}{pad}  {desc}").expect("how could it even fail");
            }
        }

        out
    }

    pub fn html(&self) -> String {
        let mut out = format!("<h2>Help Page for .{}</h2>", self.name);
        writeln!(&mut out, "{}", self.description).expect("how could it even fail");
        writeln!(&mut out, "<br><br><b>Usage</b>: {}<br>", self.usage)
            .expect("how could it even fail");

        if !self.args.is_empty() {
            writeln!(&mut out, "<h4>Arguments</h4><table>").expect("how could it even fail");

            for (tag, desc) in self.args.iter() {
                let tag = tag.as_deref().unwrap_or("(positional)");

                writeln!(&mut out, "<tr><td>{tag}</td><td>{desc}</td></tr>")
                    .expect("how could it even fail");
            }

            writeln!(&mut out, "\n</table>").expect("how could it even fail");
        }

        if !self.options.is_empty() {
            writeln!(&mut out, "<h4>Options</h4><table>").expect("how could it even fail");

            for (tag, desc) in self.options.iter() {
                writeln!(&mut out, "<tr><td>{tag}</td><td>{desc}</td></tr>")
                    .expect("how could it even fail");
            }

            writeln!(&mut out, "\n</table>").expect("how could it even fail");
        }

        out
    }
}
