//! IRC slash-command parsing (pure — unit-tested without network).

/// A parsed slash command (or plain chat text).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SlashCommand {
    Join {
        channels: Vec<String>,
    },
    Part {
        target: Option<String>,
    },
    Msg {
        target: String,
        body: String,
    },
    Nick {
        nick: String,
    },
    Me {
        action: String,
    },
    Whois {
        nick: String,
    },
    Away {
        message: String,
    },
    Back,
    Topic {
        text: Option<String>,
    },
    Ignore {
        nick: String,
    },
    Unignore {
        nick: String,
    },
    Clear,
    List,
    Help,
    /// Unknown command name (without leading slash).
    Unknown {
        name: String,
    },
    /// Not a slash command — send as plain message body.
    Plain(String),
}

/// Parse a raw composer line into a slash command or plain text.
pub fn parse_slash_command(input: &str) -> SlashCommand {
    let text = input.trim();
    if text.is_empty() {
        return SlashCommand::Plain(String::new());
    }
    if !text.starts_with('/') {
        return SlashCommand::Plain(text.to_string());
    }

    let rest = &text[1..];
    let (cmd, args) = match rest.split_once(char::is_whitespace) {
        Some((c, a)) => (c.to_ascii_lowercase(), a.trim()),
        None => (rest.to_ascii_lowercase(), ""),
    };

    match cmd.as_str() {
        "join" | "j" => {
            let channels = if args.is_empty() {
                Vec::new()
            } else {
                // Support both comma-separated and space-separated channel lists.
                let raw = args.replace(' ', ",");
                crate::channels::parse_join_command_multi(&raw)
            };
            SlashCommand::Join { channels }
        }
        "part" | "leave" => {
            let target = if args.is_empty() {
                None
            } else {
                Some(args.split_whitespace().next().unwrap_or(args).to_string())
            };
            SlashCommand::Part { target }
        }
        "msg" | "query" => match parse_msg_command(&format!("/msg {args}")) {
            Some((target, body)) => SlashCommand::Msg { target, body },
            None => SlashCommand::Unknown { name: cmd },
        },
        "nick" => {
            let nick = args.split_whitespace().next().unwrap_or("").to_string();
            if nick.is_empty() {
                SlashCommand::Unknown { name: cmd }
            } else {
                SlashCommand::Nick { nick }
            }
        }
        "me" => {
            if args.is_empty() {
                SlashCommand::Unknown { name: cmd }
            } else {
                // Full rest of line — not a single word.
                SlashCommand::Me {
                    action: args.to_string(),
                }
            }
        }
        "whois" => {
            let nick = args.split_whitespace().next().unwrap_or("").to_string();
            if nick.is_empty() {
                SlashCommand::Unknown { name: cmd }
            } else {
                SlashCommand::Whois { nick }
            }
        }
        "away" => SlashCommand::Away {
            message: if args.is_empty() {
                "Away".to_string()
            } else {
                args.to_string()
            },
        },
        "back" => SlashCommand::Back,
        "topic" => {
            if args.is_empty() {
                SlashCommand::Topic { text: None }
            } else {
                SlashCommand::Topic {
                    text: Some(args.to_string()),
                }
            }
        }
        "ignore" => {
            let nick = args.split_whitespace().next().unwrap_or("").to_string();
            if nick.is_empty() {
                SlashCommand::Unknown { name: cmd }
            } else {
                SlashCommand::Ignore { nick }
            }
        }
        "unignore" => {
            let nick = args.split_whitespace().next().unwrap_or("").to_string();
            if nick.is_empty() {
                SlashCommand::Unknown { name: cmd }
            } else {
                SlashCommand::Unignore { nick }
            }
        }
        "clear" => SlashCommand::Clear,
        "list" => SlashCommand::List,
        "help" => SlashCommand::Help,
        other => SlashCommand::Unknown {
            name: other.to_string(),
        },
    }
}

/// Parse a raw slash command string into a target and body for /msg.
pub fn parse_msg_command(input: &str) -> Option<(String, String)> {
    let rest = input
        .trim()
        .strip_prefix("/msg ")
        .or_else(|| input.trim().strip_prefix("/query "))?;
    let mut parts = rest.splitn(2, char::is_whitespace);
    let target = parts.next()?.trim().to_string();
    let body = parts.next()?.trim().to_string();
    if target.is_empty() || body.is_empty() {
        return None;
    }
    Some((target, body))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_msg_keeps_spaces_in_body() {
        assert_eq!(
            parse_msg_command("/msg alice hello there friend"),
            Some(("alice".into(), "hello there friend".into()))
        );
    }

    #[test]
    fn parse_me_takes_full_rest_of_line() {
        match parse_slash_command("/me waves at the channel") {
            SlashCommand::Me { action } => assert_eq!(action, "waves at the channel"),
            other => panic!("expected Me, got {other:?}"),
        }
    }

    #[test]
    fn parse_join_space_and_comma() {
        match parse_slash_command("/join #a #b") {
            SlashCommand::Join { channels } => {
                assert_eq!(channels, vec!["#a", "#b"]);
            }
            other => panic!("expected Join, got {other:?}"),
        }
        match parse_slash_command("/join #foo,#bar") {
            SlashCommand::Join { channels } => {
                assert_eq!(channels, vec!["#foo", "#bar"]);
            }
            other => panic!("expected Join, got {other:?}"),
        }
    }

    #[test]
    fn unknown_command_is_reported() {
        match parse_slash_command("/frobnicate") {
            SlashCommand::Unknown { name } => assert_eq!(name, "frobnicate"),
            other => panic!("expected Unknown, got {other:?}"),
        }
    }

    #[test]
    fn plain_message() {
        assert_eq!(
            parse_slash_command("hello world"),
            SlashCommand::Plain("hello world".into())
        );
    }

    #[test]
    fn help_and_clear() {
        assert_eq!(parse_slash_command("/help"), SlashCommand::Help);
        assert_eq!(parse_slash_command("/clear"), SlashCommand::Clear);
    }
}
