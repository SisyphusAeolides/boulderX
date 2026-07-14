use gtk::prelude::*;
use relm4::{gtk, ComponentSender};
use crate::app::{AppInput, AppModel};

/// Build the message composer bar (Element X-style bottom input area).
pub fn build_composer(sender: &ComponentSender<AppModel>) -> gtk::Box {
    let hbox = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(8)
        .margin_start(12)
        .margin_end(12)
        .margin_top(6)
        .margin_bottom(10)
        .build();
    hbox.add_css_class("composer");

    let entry = gtk::Entry::builder()
        .placeholder_text("Message…")
        .hexpand(true)
        .build();
    entry.add_css_class("composer-entry");

    let send_btn = gtk::Button::builder()
        .label("➤")
        .tooltip_text("Send (Enter)")
        .build();
    send_btn.add_css_class("suggested-action");
    send_btn.add_css_class("composer-send");

    let s1 = sender.clone();
    let entry_clone = entry.clone();
    entry.connect_activate(move |e| {
        let text = e.text().to_string();
        if !text.is_empty() {
            e.set_text("");
            s1.input(AppInput::SendMessage(text));
        }
    });

    let s2 = sender.clone();
    send_btn.connect_clicked(move |_| {
        let text = entry_clone.text().to_string();
        if !text.is_empty() {
            entry_clone.set_text("");
            s2.input(AppInput::SendMessage(text));
        }
    });

    hbox.append(&entry);
    hbox.append(&send_btn);
    hbox
}
