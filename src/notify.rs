use gtk::gio::{Notification, ThemedIcon};
use gtk::prelude::ApplicationExt;

pub const APP_ID: &str = "org.Sisyphus.BoulderRelay";

pub fn setup_application_icon() {
    gtk::Window::set_default_icon_name("boulder-relay");
}

pub fn send_message_notification(channel: &str, user: &str, body: &str) {
    let preview = if body.chars().count() > 180 {
        let mut truncated = String::new();
        for (index, ch) in body.chars().enumerate() {
            if index >= 177 {
                truncated.push_str("...");
                break;
            }
            truncated.push(ch);
        }
        truncated
    } else {
        body.to_string()
    };

    let notification = Notification::new(channel);
    notification.set_body(Some(&format!("{user}: {preview}")));
    notification.set_icon(&ThemedIcon::new("boulder-relay"));
    notification.set_default_action("app.activate");

    let id = format!(
        "boulder-relay-{}-{}",
        channel.replace(['#', '&', '+', '!'], ""),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0)
    );
    relm4::main_application().send_notification(Some(&id), &notification);
}