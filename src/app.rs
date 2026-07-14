use relm4::{gtk, ComponentParts, ComponentSender, SimpleComponent};
use gtk::prelude::*;
use relm4::RelmWidgetExt;
use adw::prelude::*;
use irc::client::Sender as IrcSender;
use std::sync::Arc;
use tokio::sync::mpsc;

use crate::matrix::client::{MatrixClient, MatrixEvent};
use crate::matrix::sync::bridge_matrix_events;

// ── Constants ─────────────────────────────────────────────────────────────────
pub const DEFAULT_PORT: u16 = 6697;

// ── Protocol tag ──────────────────────────────────────────────────────────────
#[derive(Debug, Clone, PartialEq)]
pub enum Protocol {
    Irc,
    Matrix { room_id: String },
}

// ── AppInput ──────────────────────────────────────────────────────────────────
#[derive(Debug)]
pub enum AppInput {
    // IRC connection lifecycle
    NetworkConnected(IrcSender),
    NetworkStatus(String),

    // IRC / Matrix unified message
    ReceiveMessage { channel: String, user: String, body: String, protocol: Protocol },
    ReceiveServerMessage(String),

    // IRC user events
    UserJoined { channel: String, user: String },
    UserLeft   { channel: String, user: String },
    UserRenamed { old: String, new: String },
    UserQuit   { user: String },

    // IRC channel list
    BatchAddUsers    { channel: String, users: Vec<String> },
    ChannelListEntry { name: String, users: u32, topic: String },
    ChannelListEnd,
    ChannelTopic     { channel: String, topic: String },

    // Sidebar actions
    SelectChannel(String),
    ToggleFavorite(String),
    PartChannel(String),

    // Matrix lifecycle
    MatrixConnected  { user_id: String },
    MatrixRoomJoined { room_id: String, room_name: String },
    MatrixRoomLeft   { room_id: String },

    // High-level commands
    MatrixLogin { homeserver: String, username: String, password: String },
    MatrixJoinRoom(String),
    MatrixEvent(MatrixEvent),
    SelectRoom(String),
    SendMessage(String),
    ShowMatrixLogin,
    ShowMatrixJoinRoom,
    ConnectIrc { server: String, port: u16, nick: String, channel: String },
    IrcMessage(String),
}

// ── AppModel ──────────────────────────────────────────────────────────────────
pub struct AppModel {
    pub matrix_client: Option<Arc<MatrixClient>>,
    pub active_room: Option<String>,
}

#[relm4::component(pub)]
impl SimpleComponent for AppModel {
    type Input  = AppInput;
    type Output = ();
    type Init   = ();

    view! {
        adw::ApplicationWindow {
            set_title: Some("boulderX"),
            set_default_size: (1100, 700),

            #[wrap(Some)]
            set_content = &gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_width_request: 240,
                },

                gtk::Separator {
                    set_orientation: gtk::Orientation::Vertical,
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_hexpand: true,

                    gtk::ScrolledWindow {
                        set_vexpand: true,
                        gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            set_margin_all: 12,
                        }
                    },

                    gtk::Box {
                        set_orientation: gtk::Orientation::Horizontal,
                        set_margin_all: 8,
                    },
                },
            },
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = AppModel {
            matrix_client: None,
            active_room: None,
        };
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, input: Self::Input, sender: ComponentSender<Self>) {
        match input {
            AppInput::MatrixLogin { homeserver, username, password } => {
                let sender2 = sender.clone();
                tokio::spawn(async move {
                    match MatrixClient::new(&homeserver).await {
                        Ok(client) => {
                            if let Err(e) = client.login_password(&username, &password).await {
                                sender2.input(AppInput::ReceiveServerMessage(
                                    format!("[Matrix login error]: {e}")
                                ));
                                return;
                            }
                            let (tx, rx) = mpsc::unbounded_channel::<MatrixEvent>();
                            bridge_matrix_events(rx, sender2.clone());
                            client.start_sync(tx);
                        }
                        Err(e) => {
                            sender2.input(AppInput::ReceiveServerMessage(
                                format!("[Matrix connect error]: {e}")
                            ));
                        }
                    }
                });
            }
            AppInput::MatrixJoinRoom(alias) => {
                if let Some(client) = &self.matrix_client {
                    let client = Arc::clone(client);
                    tokio::spawn(async move {
                        if let Ok(alias_or_id) = alias.as_str().try_into() {
                            let _: matrix_sdk::ruma::OwnedRoomOrAliasId = alias_or_id;
                            let _ = client.inner.join_room_by_id_or_alias(
                                alias.as_str().try_into().unwrap(),
                                &[]
                            ).await;
                        }
                    });
                }
            }
            AppInput::SelectRoom(room_id) | AppInput::SelectChannel(room_id) => {
                self.active_room = Some(room_id);
            }
            AppInput::SendMessage(body) => {
                if let (Some(client), Some(room_id)) = (&self.matrix_client, &self.active_room) {
                    let client = Arc::clone(client);
                    let room_id: matrix_sdk::ruma::OwnedRoomId =
                        room_id.as_str().try_into().unwrap();
                    tokio::spawn(async move {
                        let _ = client.send_message(&room_id, &body).await;
                    });
                }
            }
            // All remaining variants are handled by UI / logged and ignored here
            AppInput::NetworkConnected(_)
            | AppInput::NetworkStatus(_)
            | AppInput::ReceiveMessage { .. }
            | AppInput::ReceiveServerMessage(_)
            | AppInput::UserJoined { .. }
            | AppInput::UserLeft { .. }
            | AppInput::UserRenamed { .. }
            | AppInput::UserQuit { .. }
            | AppInput::BatchAddUsers { .. }
            | AppInput::ChannelListEntry { .. }
            | AppInput::ChannelListEnd
            | AppInput::ChannelTopic { .. }
            | AppInput::ToggleFavorite(_)
            | AppInput::PartChannel(_)
            | AppInput::MatrixConnected { .. }
            | AppInput::MatrixRoomJoined { .. }
            | AppInput::MatrixRoomLeft { .. }
            | AppInput::MatrixEvent(_)
            | AppInput::ShowMatrixLogin
            | AppInput::ShowMatrixJoinRoom
            | AppInput::ConnectIrc { .. }
            | AppInput::IrcMessage(_) => {}
        }
    }
}
