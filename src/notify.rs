use gtk::gio::{Notification, ThemedIcon};
use gtk::prelude::ApplicationExt;

pub const APP_ID: &str = "io.github.sisyphusaeolides.boulderx";

pub fn setup_application_icon() {
    gtk::Window::set_default_icon_name("boulderX");
}

/// Truncate a string to at most `max_chars` Unicode scalar values.
/// Appends "\u{2026}" (ellipsis) if truncated.
fn truncate_preview(body: &str, max_chars: usize) -> String {
    let mut chars = body.chars();
    let mut out = String::new();
    let mut count = 0;
    loop {
        match chars.next() {
            Some(ch) if count < max_chars => {
                out.push(ch);
                count += 1;
            }
            None => break,
            _ => {
                out.push('\u{2026}');
                break;
            }
        }
    }
    out
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotifyKind {
    /// A direct message from another user
    DirectMessage,
    /// A mention (@nick) in a channel
    Mention,
    /// Any other channel activity (fallback)
    Activity,
}

pub fn send_message_notification(channel: &str, user: &str, body: &str, kind: NotifyKind) {
    let preview = truncate_preview(body, 180);

    let title = match kind {
        NotifyKind::DirectMessage => format!("DM from {user}"),
        NotifyKind::Mention => format!("Mention in {channel}"),
        NotifyKind::Activity => channel.to_string(),
    };

    let notification = Notification::new(&title);
    notification.set_body(Some(&format!("{user}: {preview}")));
    notification.set_icon(&ThemedIcon::new("boulderX"));
    notification.set_default_action("app.activate");

    // Priority: urgent for DM/mention, normal otherwise
    notification.set_priority(match kind {
        NotifyKind::DirectMessage | NotifyKind::Mention => gtk::gio::NotificationPriority::High,
        NotifyKind::Activity => gtk::gio::NotificationPriority::Normal,
    });

    let id = format!(
        "boulderX-{}-{}",
        channel
            .chars()
            .filter(|c| c.is_alphanumeric())
            .collect::<String>(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    );
    relm4::main_application().send_notification(Some(&id), &notification);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_short_string_unchanged() {
        assert_eq!(truncate_preview("hello", 180), "hello");
    }

    #[test]
    fn truncate_long_string_appends_ellipsis() {
        let long: String = "a".repeat(200);
        let result = truncate_preview(&long, 180);
        assert!(result.ends_with('\u{2026}'));
        assert!(result.chars().count() <= 181);
    }

    #[test]
    fn truncate_exactly_at_boundary() {
        let exact: String = "b".repeat(180);
        let result = truncate_preview(&exact, 180);
        assert_eq!(result, exact);
    }
}
