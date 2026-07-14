use gtk::prelude::*;
use relm4::{gtk, ComponentSender};
use crate::app::{AppInput, AppModel, Protocol};
use crate::channels;

/// Protocol badge rendered next to room/channel names in the sidebar.
pub fn protocol_badge(protocol: &Protocol) -> gtk::Label {
    let (text, css) = match protocol {
        Protocol::Irc => ("IRC", "badge-irc"),
        Protocol::Matrix { .. } => ("MX", "badge-matrix"),
    };
    let label = gtk::Label::builder()
        .label(text)
        .halign(gtk::Align::Start)
        .build();
    label.add_css_class("protocol-badge");
    label.add_css_class(css);
    label
}

/// Build a single sidebar row for a channel or Matrix room.
pub fn build_room_row(
    sender: &ComponentSender<AppModel>,
    name: &str,
    unread: u32,
    is_active: bool,
    protocol: Protocol,
    is_favorite: bool,
) -> gtk::ListBoxRow {
    let row = gtk::ListBoxRow::new();
    if is_active {
        row.add_css_class("room-row-active");
    }
    row.add_css_class("room-row");

    let hbox = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(6)
        .margin_start(8)
        .margin_end(8)
        .margin_top(4)
        .margin_bottom(4)
        .build();

    // Avatar / icon placeholder
    let avatar = gtk::Label::builder()
        .label(&name.chars().next().unwrap_or('#').to_uppercase().to_string())
        .build();
    avatar.add_css_class("room-avatar");
    hbox.append(&avatar);

    // Name + protocol badge stack
    let vbox = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(1)
        .hexpand(true)
        .build();

    let name_label = gtk::Label::builder()
        .label(name)
        .halign(gtk::Align::Start)
        .ellipsize(gtk::pango::EllipsizeMode::End)
        .build();
    name_label.add_css_class("room-name");
    vbox.append(&name_label);
    vbox.append(&protocol_badge(&protocol));
    hbox.append(&vbox);

    // Unread badge
    if unread > 0 {
        let badge = gtk::Label::builder()
            .label(&unread.to_string())
            .halign(gtk::Align::End)
            .build();
        badge.add_css_class("unread-badge");
        hbox.append(&badge);
    }

    // Favorite star
    let fav_btn = gtk::Button::with_label(if is_favorite { "★" } else { "☆" });
    fav_btn.add_css_class("fav-btn");
    let s = sender.clone();
    let ch = name.to_string();
    fav_btn.connect_clicked(move |_| s.input(AppInput::ToggleFavorite(ch.clone())));
    hbox.append(&fav_btn);

    // Part/leave button for IRC channels
    if matches!(protocol, Protocol::Irc) && channels::is_channel_target(name) {
        let part_btn = gtk::Button::with_label("×");
        part_btn.add_css_class("part-btn");
        let s2 = sender.clone();
        let ch2 = name.to_string();
        part_btn.connect_clicked(move |_| s2.input(AppInput::PartChannel(ch2.clone())));
        hbox.append(&part_btn);
    }

    // Leave button for Matrix rooms
    if let Protocol::Matrix { room_id } = &protocol {
        let leave_btn = gtk::Button::with_label("×");
        leave_btn.add_css_class("part-btn");
        let s3 = sender.clone();
        let rid = room_id.clone();
        leave_btn.connect_clicked(move |_| s3.input(AppInput::MatrixRoomLeft { room_id: rid.clone() }));
        hbox.append(&leave_btn);
    }

    row.set_child(Some(&hbox));

    let s4 = sender.clone();
    let ch4 = name.to_string();
    row.connect_activate(move |_| s4.input(AppInput::SelectChannel(ch4.clone())));
    row
}

/// Section divider label for IRC / Matrix groups in the sidebar.
pub fn section_header(label: &str) -> gtk::ListBoxRow {
    let lbl = gtk::Label::builder()
        .label(label)
        .halign(gtk::Align::Start)
        .margin_start(10)
        .margin_top(10)
        .margin_bottom(2)
        .build();
    lbl.add_css_class("sidebar-section-header");
    let row = gtk::ListBoxRow::new();
    row.set_activatable(false);
    row.set_selectable(false);
    row.set_child(Some(&lbl));
    row
}
