mod channels;
mod config;
mod notify;
mod theme;

use channels::{Community, COMMUNITY_ORDER};
use config::Settings;
use futures::prelude::*;
use gtk::glib::{self, DateTime};
use gtk::prelude::*;
use irc::client::prelude::*;
use relm4::{gtk, ComponentParts, ComponentSender, RelmApp, RelmWidgetExt, SimpleComponent};
use std::collections::HashMap;
use std::thread;

const DEFAULT_SERVER: &str = "irc.libera.chat";
const DEFAULT_NICKNAME: &str = "SisyphusCode";
const DEFAULT_PORT: u16 = 6697;
const SERVER_TAB: &str = "Server";

#[derive(Copy, Clone, PartialEq, Eq)]
enum LineStyle {
    Normal,
    SelfMsg,
    System,
    Mention,
}

#[derive(Clone)]
struct ChatLine {
    text: String,
    style: LineStyle,
}

const HELP_TEXT: &str = "Commands: /join chan, /j chan, /part [#chan], /msg nick text, /clear, /nick name, /help\n\
Join box: type #channel to join, or a nick for a DM. Any channel is supported.\n";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConnectionState {
    Offline,
    Connecting,
    Connected,
}

#[derive(Debug, Clone)]
pub enum AppInput {
    UpdateNickname(String),
    UpdateServer(String),
    UpdatePassword(String),
    Connect,
    Disconnect,
    NetworkStatus(String),
    NetworkConnected(irc::client::Sender),
    SelectChannel(String),
    JoinChannel(String),
    PartChannel(String),
    ClearChannel(String),
    ToggleFavorite(String),
    ToggleMute { channel: String, user: String },
    ReceiveMessage { channel: String, user: String, body: String },
    ReceiveServerMessage(String),
    BatchAddUsers { channel: String, users: Vec<String> },
    UserJoined { channel: String, user: String },
    UserLeft { channel: String, user: String },
    UserQuit { user: String },
    SendMessage(String),
    JoinEntry(String),
    UpdateNotificationsEnabled(bool),
    UpdateBackgroundOnClose(bool),
    Quit,
    SaveSettings,
}

struct AppModel {
    connection: ConnectionState,
    status: String,
    active_channel: String,
    channels: Vec<String>,
    favorite_channels: Vec<String>,
    muted_users: HashMap<String, Vec<String>>,
    chat_histories: HashMap<String, Vec<ChatLine>>,
    channel_users: HashMap<String, Vec<String>>,
    irc_sender: Option<irc::client::Sender>,
    nickname: String,
    server: String,
    password: String,
    channel_box: gtk::Box,
    user_box: gtk::Box,
    chat_view: gtk::TextView,
    window: gtk::Window,
    notifications_enabled: bool,
    background_on_close: bool,
}

impl AppModel {
    fn normalized_nick(user: &str) -> String {
        user.trim_start_matches(&['@', '+', '%', '~', '&'][..]).to_string()
    }

    fn is_muted(&self, channel: &str, user: &str) -> bool {
        let clean = Self::normalized_nick(user);
        self.muted_users
            .get(channel)
            .map(|users| users.iter().any(|u| u == &clean))
            .unwrap_or(false)
    }

    fn timestamp_prefix() -> String {
        DateTime::now_local()
            .map(|dt| format!("[{}] ", dt.format("%H:%M").unwrap_or_default()))
            .unwrap_or_else(|_| String::from("[??:??] "))
    }

    fn extra_channels(&self) -> Vec<String> {
        let defaults: std::collections::HashSet<_> =
            channels::default_channel_names().into_iter().collect();
        self.channels
            .iter()
            .filter(|channel| {
                **channel != SERVER_TAB && !defaults.contains(*channel)
            })
            .cloned()
            .collect()
    }

    fn settings_snapshot(&self) -> Settings {
        Settings {
            nickname: self.nickname.clone(),
            server: self.server.clone(),
            password: self.password.clone(),
            favorites: self.favorite_channels.clone(),
            extra_channels: self.extra_channels(),
            last_channel: self.active_channel.clone(),
            notifications_enabled: self.notifications_enabled,
            background_on_close: self.background_on_close,
        }
    }

    fn should_notify(&self, channel: &str, user: &str, style: LineStyle) -> bool {
        if !self.notifications_enabled || user == "System" || style == LineStyle::SelfMsg {
            return false;
        }

        let hidden = !self.window.is_visible();
        let inactive = channel != self.active_channel;
        let dm = !channels::is_channel_target(channel);
        let mention = style == LineStyle::Mention;

        hidden || inactive || dm || mention
    }

    fn send_irc_join(&self, target: &str) {
        if let Some(irc_tx) = &self.irc_sender {
            if channels::is_channel_target(target) {
                let _ = irc_tx.send_join(target);
            }
        }
    }

    fn style_tag(style: LineStyle) -> &'static str {
        match style {
            LineStyle::Normal => "normal",
            LineStyle::SelfMsg => "self-msg",
            LineStyle::System => "system",
            LineStyle::Mention => "mention",
        }
    }

    fn setup_chat_tags(view: &gtk::TextView) {
        let buffer = view.buffer();
        let table = buffer.tag_table();

        for (name, fg, bg) in [
            ("normal", "#ebdbb2", None),
            ("self-msg", "#10B981", None),
            ("system", "#928374", None),
            ("mention", "#fe8019", Some("#3c3836")),
        ] {
            let tag = gtk::TextTag::new(Some(name));
            tag.set_foreground(Some(fg));
            if let Some(bg) = bg {
                tag.set_background(Some(bg));
            }
            table.add(&tag);
        }
    }

    fn message_style(&self, user: &str, body: &str) -> LineStyle {
        if user == "System" {
            return LineStyle::System;
        }
        let clean = Self::normalized_nick(user);
        if clean.eq_ignore_ascii_case(&self.nickname) {
            return LineStyle::SelfMsg;
        }
        if body.contains(&self.nickname) {
            return LineStyle::Mention;
        }
        LineStyle::Normal
    }

    fn append_line(&mut self, channel: &str, line: &str, style: LineStyle) {
        let history = self
            .chat_histories
            .entry(channel.to_string())
            .or_insert_with(Vec::new);
        history.push(ChatLine {
            text: line.to_string(),
            style,
        });

        if self.active_channel == channel {
            self.append_styled_to_chat_view(line, style);
        }
    }

    fn append_styled_to_chat_view(&self, line: &str, style: LineStyle) {
        let buffer = self.chat_view.buffer();
        let mut end = buffer.end_iter();
        buffer.insert_with_tags_by_name(&mut end, line, &[Self::style_tag(style)]);

        let mark = buffer.create_mark(None, &buffer.end_iter(), false);
        self.chat_view.scroll_to_mark(&mark, 0.0, false, 0.0, 0.0);
    }

    fn append_to_chat_view(&self, line: &str) {
        self.append_styled_to_chat_view(line, LineStyle::Normal);
    }

    fn persist_settings(&self) {
        if let Err(error) = self.settings_snapshot().save() {
            eprintln!("Failed to save Boulder Relay settings: {error}");
        }
    }

    fn show_channel_history(&self) {
        let buffer = self.chat_view.buffer();
        buffer.set_text("");

        if let Some(lines) = self.chat_histories.get(&self.active_channel) {
            for line in lines {
                self.append_styled_to_chat_view(&line.text, line.style);
            }
        }
    }

    fn append_section_header(&self, label: &str) {
        let header = gtk::Label::builder()
            .label(label)
            .halign(gtk::Align::Start)
            .margin_start(8)
            .margin_top(8)
            .margin_bottom(2)
            .build();
        header.add_css_class("channel-section");
        self.channel_box.append(&header);
    }

    fn append_channel_row(&self, sender: &ComponentSender<Self>, channel: &str) {
        let is_favorite = self.favorite_channels.iter().any(|c| c == channel);

        let row = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(4)
            .build();

        let select_btn = gtk::Button::with_label(channel);
        select_btn.set_hexpand(true);
        select_btn.set_halign(gtk::Align::Fill);

        if let Some(info) = channels::channel_info(channel) {
            select_btn.set_tooltip_text(Some(info.description));
            select_btn.add_css_class(channels::community_css_class(info.community));
        }

        let s1 = sender.clone();
        let ch1 = channel.to_string();
        select_btn.connect_clicked(move |_| {
            s1.input(AppInput::SelectChannel(ch1.clone()));
        });

        let fav_icon = if is_favorite { "★" } else { "☆" };
        let fav_btn = gtk::Button::with_label(fav_icon);
        fav_btn.add_css_class("fav-btn");
        fav_btn.set_tooltip_text(Some(if is_favorite {
            "Remove from favorites"
        } else {
            "Add to favorites"
        }));

        let s2 = sender.clone();
        let ch2 = channel.to_string();
        fav_btn.connect_clicked(move |_| {
            s2.input(AppInput::ToggleFavorite(ch2.clone()));
        });

        row.append(&select_btn);
        row.append(&fav_btn);

        if channel.starts_with('#') {
            let part_btn = gtk::Button::with_label("×");
            part_btn.add_css_class("part-btn");
            part_btn.set_tooltip_text(Some("Leave channel"));

            let s3 = sender.clone();
            let ch3 = channel.to_string();
            part_btn.connect_clicked(move |_| {
                s3.input(AppInput::PartChannel(ch3.clone()));
            });

            row.append(&part_btn);
        }

        self.channel_box.append(&row);
    }

    fn refresh_channels(&self, sender: &ComponentSender<Self>) {
        while let Some(child) = self.channel_box.first_child() {
            self.channel_box.remove(&child);
        }

        let mut favorites = Vec::new();
        let mut by_community: std::collections::HashMap<Community, Vec<String>> =
            std::collections::HashMap::new();
        let mut other_channels = Vec::new();

        for channel in &self.channels {
            if self.favorite_channels.contains(channel) {
                favorites.push(channel.clone());
                continue;
            }

            if let Some(community) = channels::community_for(channel) {
                by_community
                    .entry(community)
                    .or_default()
                    .push(channel.clone());
            } else {
                other_channels.push(channel.clone());
            }
        }

        if !favorites.is_empty() {
            self.append_section_header("★ Favorites");
            for channel in &favorites {
                self.append_channel_row(sender, channel);
            }
        }

        for community in COMMUNITY_ORDER {
            let Some(mut community_channels) = by_community.remove(community) else {
                continue;
            };
            community_channels.sort_by_key(|name| name.to_lowercase());
            self.append_section_header(channels::community_label(*community));
            for channel in &community_channels {
                self.append_channel_row(sender, channel);
            }
        }

        if !other_channels.is_empty() {
            other_channels.sort_by_key(|name| name.to_lowercase());
            self.append_section_header("Other");
            for channel in &other_channels {
                self.append_channel_row(sender, channel);
            }
        }
    }

    fn refresh_users(&self, sender: &ComponentSender<Self>) {
        while let Some(child) = self.user_box.first_child() {
            self.user_box.remove(&child);
        }

        if let Some(users) = self.channel_users.get(&self.active_channel) {
            for user in users {
                let clean_user = Self::normalized_nick(user);
                let muted = self.is_muted(&self.active_channel, user);

                let row = gtk::Box::builder()
                    .orientation(gtk::Orientation::Horizontal)
                    .spacing(4)
                    .build();

                let dm_btn = gtk::Button::with_label(user);
                dm_btn.set_hexpand(true);
                dm_btn.set_halign(gtk::Align::Fill);
                dm_btn.add_css_class("user-btn");
                if muted {
                    dm_btn.add_css_class("muted-user");
                }

                let s1 = sender.clone();
                let u1 = clean_user.clone();
                dm_btn.connect_clicked(move |_| {
                    s1.input(AppInput::JoinChannel(u1.clone()));
                });

                let mute_icon = if muted { "🔇" } else { "🔊" };
                let mute_btn = gtk::Button::with_label(mute_icon);
                mute_btn.add_css_class("mute-btn");

                let s2 = sender.clone();
                let c2 = self.active_channel.clone();
                let u2 = clean_user.clone();
                mute_btn.connect_clicked(move |_| {
                    s2.input(AppInput::ToggleMute {
                        channel: c2.clone(),
                        user: u2.clone(),
                    });
                });

                row.append(&dm_btn);
                row.append(&mute_btn);
                self.user_box.append(&row);
            }
        }
    }
}

#[relm4::component]
impl SimpleComponent for AppModel {
    type Init = ();
    type Input = AppInput;
    type Output = ();

    view! {
        gtk::Window {
            set_default_size: (1200, 700),
            add_css_class: "boulder-relay",

            connect_close_request[sender] => move |window| {
                sender.input(AppInput::SaveSettings);
                if model.background_on_close && model.connection == ConnectionState::Connected {
                    window.set_visible(false);
                    glib::Propagation::Stop
                } else {
                    glib::Propagation::Proceed
                }
            },

            gtk::Paned {
                set_orientation: gtk::Orientation::Horizontal,
                set_position: 240,

                #[wrap(Some)]
                set_start_child = &gtk::Box {
                    set_orientation: gtk::Orientation::Vertical, set_spacing: 12, set_width_request: 200,
                    add_css_class: "sidebar", set_margin_all: 0,

                    gtk::Label { set_label: "BOULDER RELAY", add_css_class: "sidebar-title", set_margin_top: 16 },
                    gtk::Label { set_label: "Fedora · RHEL · Rocky on Libera", set_margin_start: 12, set_margin_end: 12 },
                    gtk::Separator { set_orientation: gtk::Orientation::Horizontal },

                    gtk::Label { set_label: "Network Configuration", add_css_class: "sidebar-subtitle", set_halign: gtk::Align::Start, set_margin_start: 12 },
                    gtk::Entry {
                        set_text: &model.nickname, set_placeholder_text: Some("Nickname"), set_margin_start: 12, set_margin_end: 12,
                        connect_changed[sender] => move |entry| { sender.input(AppInput::UpdateNickname(entry.text().to_string())); }
                    },
                    gtk::Box {
                        set_orientation: gtk::Orientation::Horizontal,
                        set_spacing: 4,
                        set_margin_start: 12,
                        set_margin_end: 12,
                        gtk::Entry {
                            set_text: &model.password,
                            set_placeholder_text: Some("NickServ Password (Opt)"),
                            set_hexpand: true,
                            set_visibility: false,
                            connect_changed[sender] => move |entry| { sender.input(AppInput::UpdatePassword(entry.text().to_string())); }
                        },
                        gtk::Button {
                            set_label: "👁",
                            set_tooltip_text: Some("Show or hide password"),
                            connect_clicked => move |button| {
                                if let Some(entry) = button.prev_sibling().and_downcast::<gtk::Entry>() {
                                    let visible = entry.property::<bool>("visibility");
                                    entry.set_visibility(!visible);
                                }
                            }
                        },
                    },
                    gtk::Entry {
                        set_text: &model.server, set_placeholder_text: Some("Server address"), set_margin_start: 12, set_margin_end: 12,
                        connect_changed[sender] => move |entry| { sender.input(AppInput::UpdateServer(entry.text().to_string())); }
                    },
                    gtk::Label {
                        #[watch]
                        set_label: &format!("Status: {}", model.status),
                        set_ellipsize: gtk::pango::EllipsizeMode::End,
                        set_margin_start: 8,
                        set_margin_end: 8,
                        add_css_class: match model.connection {
                            ConnectionState::Connected => "status-connected",
                            ConnectionState::Connecting => "status-connecting",
                            ConnectionState::Offline => "status-offline",
                        },
                    },
                    gtk::Box {
                        set_orientation: gtk::Orientation::Horizontal,
                        set_spacing: 8,
                        set_margin_start: 12,
                        set_margin_end: 12,
                        gtk::Button {
                            set_label: "Connect",
                            set_sensitive: model.connection == ConnectionState::Offline,
                            connect_clicked => AppInput::Connect,
                        },
                        gtk::Button {
                            set_label: "Disconnect",
                            add_css_class: "destructive",
                            set_sensitive: model.connection == ConnectionState::Connected,
                            connect_clicked => AppInput::Disconnect,
                        },
                        gtk::Button {
                            set_label: "Quit",
                            add_css_class: "destructive",
                            connect_clicked => AppInput::Quit,
                        },
                    },
                    gtk::CheckButton {
                        set_label: Some("Run in background when closed"),
                        set_active: model.background_on_close,
                        set_margin_start: 12,
                        set_margin_end: 12,
                        connect_toggled[sender] => move |check| {
                            sender.input(AppInput::UpdateBackgroundOnClose(check.is_active()));
                        }
                    },
                    gtk::CheckButton {
                        set_label: Some("Desktop notifications"),
                        set_active: model.notifications_enabled,
                        set_margin_start: 12,
                        set_margin_end: 12,
                        connect_toggled[sender] => move |check| {
                            sender.input(AppInput::UpdateNotificationsEnabled(check.is_active()));
                        }
                    },

                    gtk::Separator { set_orientation: gtk::Orientation::Horizontal },

                    gtk::Label { set_label: "Channels & DMs", add_css_class: "sidebar-subtitle", set_halign: gtk::Align::Start, set_margin_start: 12 },
                    gtk::Entry {
                        set_placeholder_text: Some("#channel to join, nick for DM, or /join …"),
                        set_margin_start: 12,
                        set_margin_end: 12,
                        connect_activate[sender] => move |entry| {
                            let text = entry.text().to_string();
                            if !text.is_empty() {
                                entry.set_text("");
                                sender.input(AppInput::JoinEntry(text));
                            }
                        }
                    },

                    gtk::ScrolledWindow {
                        set_vexpand: true, set_hexpand: true,
                        #[local_ref] channel_box_ref -> gtk::Box {}
                    }
                },

                #[wrap(Some)]
                set_end_child = &gtk::Paned {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_position: 680,

                    #[wrap(Some)]
                    set_start_child = &gtk::Box {
                        set_orientation: gtk::Orientation::Vertical, set_spacing: 12, set_margin_all: 16, set_width_request: 300,
                        add_css_class: "chat-panel",

                        gtk::Label { #[watch] set_label: &format!("Active Context: {}", model.active_channel), set_halign: gtk::Align::Start },

                        gtk::ScrolledWindow {
                            set_vexpand: true, set_hexpand: true, set_propagate_natural_height: true,
                            #[local_ref] chat_view_ref -> gtk::TextView {
                                set_editable: false,
                                set_cursor_visible: false,
                                set_wrap_mode: gtk::WrapMode::Word,
                                set_vexpand: true,
                            }
                        },

                        gtk::Entry {
                            set_placeholder_text: Some("Message, /join #chan, or /msg nick text…"), set_hexpand: true,
                            connect_activate[sender] => move |entry| {
                                let text = entry.text().to_string();
                                if !text.is_empty() {
                                    entry.set_text("");
                                    sender.input(AppInput::SendMessage(text));
                                }
                            }
                        }
                    },

                    #[wrap(Some)]
                    set_end_child = &gtk::Box {
                        set_orientation: gtk::Orientation::Vertical, set_spacing: 12, set_width_request: 200,
                        add_css_class: "sidebar", set_margin_all: 0,

                        gtk::Label { set_label: "USERS IN CHANNEL", add_css_class: "sidebar-title", set_margin_top: 16, set_margin_bottom: 8 },
                        gtk::Separator { set_orientation: gtk::Orientation::Horizontal },

                        gtk::ScrolledWindow {
                            set_vexpand: true, set_hexpand: true,
                            #[local_ref] user_box_ref -> gtk::Box {}
                        }
                    }
                }
            }
        }
    }

    fn init(_init: Self::Init, root: Self::Root, sender: ComponentSender<Self>) -> ComponentParts<Self> {
        theme::attach_window(&root);

        let settings = Settings::load();
        let server_tab = String::from(SERVER_TAB);
        let default_channels = channels::default_channel_names();

        let mut chat_histories = HashMap::new();
        chat_histories.insert(
            server_tab.clone(),
            vec![
                ChatLine {
                    text: String::from(
                        "[System]: Ready for Libera.Chat. Register with NickServ and connect.\n",
                    ),
                    style: LineStyle::System,
                },
                ChatLine {
                    text: String::from(
                        "[System]: #fedora, #fedora-devel, and #rhel-devel require a registered nick.\n",
                    ),
                    style: LineStyle::System,
                },
                ChatLine {
                    text: String::from(
                        "[System]: Rocky channels also require registration (wiki.rockylinux.org/irc).\n",
                    ),
                    style: LineStyle::System,
                },
                ChatLine {
                    text: String::from(
                        "[System]: Join any channel via the sidebar (#channel) or /join channel. Custom channels are saved.\n",
                    ),
                    style: LineStyle::System,
                },
                ChatLine {
                    text: String::from(
                        "[System]: Settings are saved to ~/.config/boulder-relay/settings.conf\n",
                    ),
                    style: LineStyle::System,
                },
            ],
        );

        let channel_box = gtk::Box::new(gtk::Orientation::Vertical, 6);
        let user_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
        let chat_view = gtk::TextView::new();
        Self::setup_chat_tags(&chat_view);

        let mut channels = vec![server_tab.clone()];
        channels.extend(default_channels.clone());
        for extra in &settings.extra_channels {
            if !channels.contains(extra) {
                channels.push(extra.clone());
            }
        }

        let favorites = if settings.favorites.is_empty() {
            vec![
                server_tab.clone(),
                String::from("#rockylinux-devel"),
                String::from("#fedora-devel"),
                String::from("#rhel-devel"),
            ]
        } else {
            settings.favorites.clone()
        };

        let active_channel = if settings.last_channel.is_empty()
            || !channels.contains(&settings.last_channel)
        {
            server_tab.clone()
        } else {
            settings.last_channel
        };

        let model = AppModel {
            connection: ConnectionState::Offline,
            status: String::from("Offline"),
            active_channel,
            channels,
            favorite_channels: favorites,
            muted_users: HashMap::new(),
            chat_histories,
            channel_users: HashMap::new(),
            irc_sender: None,
            nickname: if settings.nickname.is_empty() {
                String::from(DEFAULT_NICKNAME)
            } else {
                settings.nickname
            },
            server: if settings.server.is_empty() {
                String::from(DEFAULT_SERVER)
            } else {
                settings.server
            },
            password: settings.password,
            channel_box: channel_box.clone(),
            user_box: user_box.clone(),
            chat_view: chat_view.clone(),
            window: root.clone(),
            notifications_enabled: settings.notifications_enabled,
            background_on_close: settings.background_on_close,
        };

        let channel_box_ref = &model.channel_box;
        let user_box_ref = &model.user_box;
        let chat_view_ref = &model.chat_view;
        let widgets = view_output!();

        let mut parts = ComponentParts { model, widgets };
        parts.model.show_channel_history();
        parts.model.refresh_channels(&sender);
        parts.model.refresh_users(&sender);
        parts
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            AppInput::UpdateNickname(nick) => self.nickname = nick,
            AppInput::UpdateServer(srv) => self.server = srv,
            AppInput::UpdatePassword(pwd) => self.password = pwd,
            AppInput::UpdateNotificationsEnabled(enabled) => {
                self.notifications_enabled = enabled;
                self.persist_settings();
            }
            AppInput::UpdateBackgroundOnClose(enabled) => {
                self.background_on_close = enabled;
                self.persist_settings();
            }

            AppInput::Quit => {
                self.persist_settings();
                if let Some(irc_tx) = self.irc_sender.take() {
                    let _ = irc_tx.send_quit("Boulder Relay signing off");
                }
                relm4::main_application().quit();
            }

            AppInput::SaveSettings => self.persist_settings(),

            AppInput::Connect => {
                if self.connection != ConnectionState::Offline {
                    return;
                }
                self.connection = ConnectionState::Connecting;
                self.status = String::from("Connecting...");
                self.persist_settings();

                let sender_clone = sender.clone();
                let channels_to_join: Vec<String> = self
                    .channels
                    .iter()
                    .filter(|c| channels::is_channel_target(c))
                    .cloned()
                    .collect();

                let nickname = self.nickname.clone();
                let server_addr = self.server.clone();
                let pwd = self.password.clone();

                thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().expect("Failed to build Tokio core");
                    rt.block_on(async {
                        let needs_nickserv = !pwd.is_empty();
                        let config = Config {
                            nickname: Some(nickname.clone()),
                            server: Some(server_addr),
                            channels: vec![],
                            port: Some(DEFAULT_PORT),
                            use_tls: Some(true),
                            nick_password: if needs_nickserv {
                                Some(pwd.clone())
                            } else {
                                None
                            },
                            ..Config::default()
                        };

                        let mut client = match Client::from_config(config).await {
                            Ok(c) => c,
                            Err(e) => {
                                sender_clone.input(AppInput::NetworkStatus(format!(
                                    "Connection failed: {e}"
                                )));
                                return;
                            }
                        };

                        if let Err(e) = client.identify() {
                            sender_clone.input(AppInput::NetworkStatus(format!(
                                "NickServ auth failed: {e}"
                            )));
                            return;
                        }

                        let irc_tx = client.sender();
                        sender_clone.input(AppInput::NetworkConnected(irc_tx.clone()));

                        let join_channels = |tx: &irc::client::Sender| {
                            for chan in &channels_to_join {
                                let _ = tx.send_join(chan);
                            }
                            if !channels_to_join.is_empty() {
                                sender_clone.input(AppInput::ReceiveServerMessage(format!(
                                    "[System]: Joining {} channel(s).",
                                    channels_to_join.len()
                                )));
                            }
                        };

                        let mut channels_joined = false;
                        let mut stream = match client.stream() {
                            Ok(s) => s,
                            Err(_) => return,
                        };

                        while let Some(result) = stream.next().await {
                            let message = match result {
                                Ok(m) => m,
                                Err(e) => {
                                    sender_clone.input(AppInput::ReceiveServerMessage(format!(
                                        "[Error]: {e}"
                                    )));
                                    continue;
                                }
                            };
                            let user = message
                                .source_nickname()
                                .unwrap_or("Unknown")
                                .to_string();

                            match message.command {
                                Command::PRIVMSG(target, body) => {
                                    let display_target = if target == nickname {
                                        user.clone()
                                    } else {
                                        target
                                    };
                                    sender_clone.input(AppInput::ReceiveMessage {
                                        channel: display_target,
                                        user,
                                        body,
                                    });
                                }
                                Command::JOIN(channel, _, _) => {
                                    sender_clone.input(AppInput::UserJoined {
                                        channel: channel.clone(),
                                        user: user.clone(),
                                    });
                                    sender_clone.input(AppInput::ReceiveMessage {
                                        channel,
                                        user: "System".to_string(),
                                        body: format!("{} joined.", user),
                                    });
                                }
                                Command::PART(channel, _) => {
                                    sender_clone.input(AppInput::UserLeft {
                                        channel: channel.clone(),
                                        user: user.clone(),
                                    });
                                    sender_clone.input(AppInput::ReceiveMessage {
                                        channel,
                                        user: "System".to_string(),
                                        body: format!("{} left.", user),
                                    });
                                }
                                Command::QUIT(_) => {
                                    sender_clone.input(AppInput::UserQuit { user });
                                }
                                Command::NOTICE(_, body) => {
                                    sender_clone.input(AppInput::ReceiveServerMessage(format!(
                                        "[Notice]: {body}"
                                    )));
                                    if needs_nickserv
                                        && !channels_joined
                                        && body.contains("You are now identified")
                                    {
                                        channels_joined = true;
                                        join_channels(&irc_tx);
                                    }
                                }
                                Command::Response(code, args) => {
                                    if !channels_joined {
                                        match code {
                                            Response::RPL_LOGGEDIN => {
                                                channels_joined = true;
                                                join_channels(&irc_tx);
                                            }
                                            Response::RPL_ENDOFMOTD | Response::ERR_NOMOTD
                                                if !needs_nickserv =>
                                            {
                                                channels_joined = true;
                                                join_channels(&irc_tx);
                                            }
                                            _ => {}
                                        }
                                    }

                                    if code == Response::RPL_NAMREPLY && args.len() >= 4 {
                                        let channel = args
                                            .iter()
                                            .find(|a| a.starts_with('#'))
                                            .cloned()
                                            .unwrap_or_else(|| args[2].clone());

                                        let users: Vec<String> = args
                                            .last()
                                            .unwrap_or(&String::new())
                                            .split_whitespace()
                                            .map(|s| s.to_string())
                                            .collect();

                                        sender_clone.input(AppInput::BatchAddUsers {
                                            channel,
                                            users,
                                        });
                                    } else if args.len() > 1 {
                                        sender_clone.input(AppInput::ReceiveServerMessage(
                                            format!("[{code:?}]: {}", args[1..].join(" ")),
                                        ));
                                    }
                                }
                                _ => {}
                            }
                        }

                        sender_clone.input(AppInput::NetworkStatus(String::from("Disconnected")));
                        sender_clone.input(AppInput::ReceiveServerMessage(
                            String::from("[System]: Connection closed."),
                        ));
                    });
                });
            }

            AppInput::Disconnect => {
                if let Some(irc_tx) = self.irc_sender.take() {
                    let _ = irc_tx.send_quit("Boulder Relay signing off");
                    self.connection = ConnectionState::Offline;
                    self.status = String::from("Offline");
                    self.append_line(
                        SERVER_TAB,
                        &format!(
                            "{}[System]: Disconnected by user.\n",
                            Self::timestamp_prefix()
                        ),
                        LineStyle::System,
                    );
                }
            }

            AppInput::NetworkStatus(new_status) => {
                self.status = new_status.clone();
                if new_status == "Disconnected" || new_status.starts_with("Connection failed") {
                    self.connection = ConnectionState::Offline;
                    self.irc_sender = None;
                }
            }

            AppInput::NetworkConnected(irc_tx) => {
                self.irc_sender = Some(irc_tx);
                self.connection = ConnectionState::Connected;
                self.status = String::from("Connected");
                self.append_line(
                    SERVER_TAB,
                    &format!(
                        "{}[System]: Connected to {} as {}.\n",
                        Self::timestamp_prefix(),
                        self.server,
                        self.nickname
                    ),
                    LineStyle::System,
                );
            }

            AppInput::SelectChannel(channel) => {
                self.active_channel = channel;
                self.show_channel_history();
                self.refresh_users(&sender);
                self.persist_settings();
            }

            AppInput::JoinChannel(target) => {
                if !self.channels.contains(&target) {
                    self.channels.push(target.clone());
                    self.chat_histories.insert(
                        target.clone(),
                        vec![ChatLine {
                            text: format!("[System]: Tracking {target}\n"),
                            style: LineStyle::System,
                        }],
                    );
                    self.refresh_channels(&sender);
                    self.send_irc_join(&target);
                } else {
                    self.send_irc_join(&target);
                }

                self.active_channel = target;
                self.show_channel_history();
                self.refresh_users(&sender);
                self.persist_settings();
            }

            AppInput::PartChannel(channel) => {
                if channel == SERVER_TAB || !channel.starts_with('#') {
                    return;
                }

                if let Some(irc_tx) = &self.irc_sender {
                    let _ = irc_tx.send_part(&channel);
                }

                self.channels.retain(|c| c != &channel);
                self.chat_histories.remove(&channel);
                self.channel_users.remove(&channel);
                self.muted_users.remove(&channel);

                if self.active_channel == channel {
                    self.active_channel = String::from(SERVER_TAB);
                    self.show_channel_history();
                }

                self.refresh_channels(&sender);
                self.refresh_users(&sender);
                self.persist_settings();
            }

            AppInput::ClearChannel(channel) => {
                self.chat_histories.insert(channel.clone(), Vec::new());
                if self.active_channel == channel {
                    self.chat_view.buffer().set_text("");
                }
            }

            AppInput::ToggleFavorite(channel) => {
                if self.favorite_channels.contains(&channel) {
                    self.favorite_channels.retain(|c| c != &channel);
                } else {
                    self.favorite_channels.push(channel.clone());
                }
                self.refresh_channels(&sender);
                self.persist_settings();
            }

            AppInput::ToggleMute { channel, user } => {
                let list = self
                    .muted_users
                    .entry(channel.clone())
                    .or_insert_with(Vec::new);

                if list.contains(&user) {
                    list.retain(|u| u != &user);
                    self.append_line(
                        &channel,
                        &format!(
                            "{}[System]: Unmuted {}\n",
                            Self::timestamp_prefix(),
                            user
                        ),
                        LineStyle::System,
                    );
                } else {
                    list.push(user.clone());
                    list.sort_by_key(|u| u.to_lowercase());
                    self.append_line(
                        &channel,
                        &format!(
                            "{}[System]: Muted {}\n",
                            Self::timestamp_prefix(),
                            user
                        ),
                        LineStyle::System,
                    );
                }

                if self.active_channel == channel {
                    self.refresh_users(&sender);
                }
            }

            AppInput::ReceiveMessage { channel, user, body } => {
                if self.is_muted(&channel, &user) {
                    return;
                }

                if !self.channels.contains(&channel) && !channel.starts_with('#') {
                    self.channels.push(channel.clone());
                    self.refresh_channels(&sender);
                }

                let style = self.message_style(&user, &body);
                let line = format!(
                    "{}<{}> {}\n",
                    Self::timestamp_prefix(),
                    user,
                    body
                );
                self.append_line(&channel, &line, style);

                if self.should_notify(&channel, &user, style) {
                    notify::send_message_notification(&channel, &user, &body);
                }
            }

            AppInput::ReceiveServerMessage(body) => {
                let line = format!("{}{}\n", Self::timestamp_prefix(), body);
                self.append_line(SERVER_TAB, &line, LineStyle::System);
            }

            AppInput::BatchAddUsers { channel, users } => {
                let list = self
                    .channel_users
                    .entry(channel.clone())
                    .or_insert_with(Vec::new);
                for u in users {
                    if !list.contains(&u) {
                        list.push(u);
                    }
                }
                list.sort_by_key(|a| a.to_lowercase());
                if self.active_channel == channel {
                    self.refresh_users(&sender);
                }
            }

            AppInput::UserJoined { channel, user } => {
                let list = self
                    .channel_users
                    .entry(channel.clone())
                    .or_insert_with(Vec::new);
                if !list.contains(&user) {
                    list.push(user);
                    list.sort_by_key(|a| a.to_lowercase());
                }
                if self.active_channel == channel {
                    self.refresh_users(&sender);
                }
            }

            AppInput::UserLeft { channel, user } => {
                if let Some(list) = self.channel_users.get_mut(&channel) {
                    list.retain(|u| u != &user);
                }
                if self.active_channel == channel {
                    self.refresh_users(&sender);
                }
            }

            AppInput::UserQuit { user } => {
                for list in self.channel_users.values_mut() {
                    list.retain(|u| u != &user);
                }
                self.refresh_users(&sender);
            }

            AppInput::JoinEntry(text) => {
                let text = text.trim();
                if text.is_empty() {
                    return;
                }

                if text.starts_with('/') {
                    sender.input(AppInput::SendMessage(text.to_string()));
                    return;
                }

                match channels::parse_join_entry(text) {
                    Some(channels::JoinTarget::Channel(channel)) => {
                        sender.input(AppInput::JoinChannel(channel));
                    }
                    Some(channels::JoinTarget::DirectMessage(nick)) => {
                        sender.input(AppInput::JoinChannel(nick));
                    }
                    None => {}
                }
            }

            AppInput::SendMessage(text) => {
                let text = text.trim();
                if text.is_empty() {
                    return;
                }

                if text.starts_with('/') {
                    let mut parts = text.splitn(3, ' ');
                    let command = parts.next().unwrap_or("");
                    match command {
                        "/join" | "/j" => {
                            if let Some(channel) = parts.next() {
                                if let Some(channel) = channels::parse_join_command(channel) {
                                    sender.input(AppInput::JoinChannel(channel));
                                }
                            }
                            return;
                        }
                        "/msg" | "/query" => {
                            if let Some(target) = parts.next() {
                                let body = parts.next().unwrap_or("");
                                if !body.is_empty() {
                                    if let Some(irc_tx) = &self.irc_sender {
                                        let _ = irc_tx.send_privmsg(target, body);
                                        let line = format!(
                                            "{}<{}> {}\n",
                                            Self::timestamp_prefix(),
                                            self.nickname,
                                            body
                                        );
                                        self.append_line(target, &line, LineStyle::SelfMsg);
                                    }
                                } else {
                                    sender.input(AppInput::JoinChannel(target.to_string()));
                                }
                            }
                            return;
                        }
                        "/nick" => {
                            if let Some(nick) = parts.next() {
                                self.nickname = nick.to_string();
                                self.persist_settings();
                                self.append_line(
                                    SERVER_TAB,
                                    &format!(
                                        "{}[System]: Nickname updated locally to {}. Reconnect to apply.\n",
                                        Self::timestamp_prefix(),
                                        self.nickname
                                    ),
                                    LineStyle::System,
                                );
                            }
                            return;
                        }
                        "/part" => {
                            let target = parts
                                .next()
                                .map(str::to_string)
                                .unwrap_or_else(|| self.active_channel.clone());
                            sender.input(AppInput::PartChannel(target));
                            return;
                        }
                        "/clear" => {
                            sender.input(AppInput::ClearChannel(self.active_channel.clone()));
                            return;
                        }
                        "/help" => {
                            let channel = self.active_channel.clone();
                            self.append_line(&channel, HELP_TEXT, LineStyle::System);
                            return;
                        }
                        _ => {}
                    }
                }

                if self.active_channel == SERVER_TAB {
                    self.append_line(
                        SERVER_TAB,
                        &format!(
                            "{}[System]: Select a channel or DM before sending.\n",
                            Self::timestamp_prefix()
                        ),
                        LineStyle::System,
                    );
                    return;
                }

                if let Some(irc_tx) = &self.irc_sender {
                    if irc_tx.send_privmsg(&self.active_channel, text).is_ok() {
                        let channel = self.active_channel.clone();
                        let line = format!(
                            "{}<{}> {}\n",
                            Self::timestamp_prefix(),
                            self.nickname,
                            text
                        );
                        self.append_line(&channel, &line, LineStyle::SelfMsg);
                    }
                } else {
                    let channel = self.active_channel.clone();
                    self.append_line(
                        &channel,
                        &format!(
                            "{}[System]: Cannot send message, not connected.\n",
                            Self::timestamp_prefix()
                        ),
                        LineStyle::System,
                    );
                }
            }
        }
    }
}

fn main() {
    let app = RelmApp::new(notify::APP_ID);
    theme::load_css();
    notify::setup_application_icon();
    app.run::<AppModel>(());
}
