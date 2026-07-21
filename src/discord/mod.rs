//! Discord bot gateway integration. This module authenticates only with bot tokens.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use serenity::{
    async_trait,
    client::{Client, Context, EventHandler},
    http::Http,
    model::{
        channel::{ChannelType, Message},
        gateway::{GatewayIntents, Ready},
        guild::Guild,
        id::{ChannelId, UserId},
    },
};
use tokio::sync::mpsc;

use crate::app::{AppInput, AppModel};
use crate::runtime;
use relm4::ComponentSender;

#[derive(Debug, Clone)]
pub enum DiscordEvent {
    Connected {
        user_id: String,
    },
    ChannelDiscovered {
        channel_id: String,
        display_name: String,
    },
    Message {
        channel_id: String,
        dm_display_name: Option<String>,
        sender: String,
        body: String,
    },
    Error(String),
    Disconnected,
}

#[derive(Debug, Clone)]
pub struct DiscordChannel {
    pub channel_id: String,
    pub display_name: String,
}

#[derive(Debug, Default, Clone)]
pub struct ChannelRegistry {
    channels: HashMap<String, DiscordChannel>,
}

impl ChannelRegistry {
    pub fn insert(&mut self, channel_id: String, display_name: String) {
        self.channels.insert(
            channel_id.clone(),
            DiscordChannel {
                channel_id,
                display_name,
            },
        );
    }

    pub fn get(&self, channel_id: &str) -> Option<&DiscordChannel> {
        self.channels.get(channel_id)
    }

    pub fn find_by_display_name(&self, name: &str) -> Option<&DiscordChannel> {
        self.channels
            .values()
            .find(|channel| channel.display_name == name)
    }

    pub fn all(&self) -> Vec<&DiscordChannel> {
        let mut channels: Vec<_> = self.channels.values().collect();
        channels.sort_by(|a, b| {
            a.display_name
                .to_lowercase()
                .cmp(&b.display_name.to_lowercase())
        });
        channels
    }

    pub fn clear(&mut self) {
        self.channels.clear();
    }
}

#[derive(Clone)]
pub struct DiscordClient {
    http: Arc<Http>,
    shard_manager: Arc<serenity::gateway::ShardManager>,
}

impl DiscordClient {
    /// Connect an authorized Discord bot account to the gateway.
    pub async fn connect(
        bot_token: &str,
        tx: mpsc::UnboundedSender<DiscordEvent>,
    ) -> anyhow::Result<Self> {
        if bot_token.trim().is_empty() {
            anyhow::bail!("a Discord bot token is required");
        }

        let intents = GatewayIntents::GUILDS
            | GatewayIntents::GUILD_MESSAGES
            | GatewayIntents::DIRECT_MESSAGES
            | GatewayIntents::MESSAGE_CONTENT;
        let handler = DiscordHandler {
            tx: tx.clone(),
            bot_user_id: Mutex::new(None),
        };
        let mut client = Client::builder(bot_token, intents)
            .event_handler(handler)
            .await?;
        let connected = Self {
            http: client.http.clone(),
            shard_manager: client.shard_manager.clone(),
        };

        tokio::spawn(async move {
            match client.start().await {
                Ok(()) => {
                    let _ = tx.send(DiscordEvent::Disconnected);
                }
                Err(error) => {
                    let _ = tx.send(DiscordEvent::Error(format!(
                        "Discord gateway stopped: {error}"
                    )));
                    let _ = tx.send(DiscordEvent::Disconnected);
                }
            }
        });

        Ok(connected)
    }

    pub async fn send_message(&self, channel_id: &str, body: &str) -> anyhow::Result<()> {
        let id = channel_id
            .parse::<u64>()
            .map(ChannelId::new)
            .map_err(|_| anyhow::anyhow!("invalid Discord channel id"))?;
        id.say(&self.http, body).await?;
        Ok(())
    }

    pub async fn shutdown(&self) {
        self.shard_manager.shutdown_all().await;
    }
}

struct DiscordHandler {
    tx: mpsc::UnboundedSender<DiscordEvent>,
    bot_user_id: Mutex<Option<UserId>>,
}

impl DiscordHandler {
    fn discover_guild_channels(&self, guild: &Guild) {
        let mut channels: Vec<_> = guild
            .channels
            .values()
            .filter(|channel| matches!(channel.kind, ChannelType::Text | ChannelType::News))
            .collect();
        channels.sort_by(|a, b| {
            a.position
                .cmp(&b.position)
                .then_with(|| a.name.cmp(&b.name))
        });
        for channel in channels {
            let _ = self.tx.send(DiscordEvent::ChannelDiscovered {
                channel_id: channel.id.get().to_string(),
                display_name: format!("{} / #{}", guild.name, channel.name),
            });
        }
    }
}

#[async_trait]
impl EventHandler for DiscordHandler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        if let Ok(mut bot_user_id) = self.bot_user_id.lock() {
            *bot_user_id = Some(ready.user.id);
        }
        let _ = self.tx.send(DiscordEvent::Connected {
            user_id: ready.user.name.clone(),
        });

        match ctx.http.get_user_dm_channels().await {
            Ok(channels) => {
                for channel in channels {
                    let _ = self.tx.send(DiscordEvent::ChannelDiscovered {
                        channel_id: channel.id.get().to_string(),
                        display_name: format!("DM: {}", channel.recipient.name),
                    });
                }
            }
            Err(error) => {
                let _ = self.tx.send(DiscordEvent::Error(format!(
                    "could not discover Discord DMs: {error}"
                )));
            }
        }
    }

    async fn guild_create(&self, _ctx: Context, guild: Guild, _is_new: Option<bool>) {
        self.discover_guild_channels(&guild);
    }

    async fn channel_create(&self, _ctx: Context, channel: serenity::model::channel::GuildChannel) {
        if matches!(channel.kind, ChannelType::Text | ChannelType::News) {
            let _ = self.tx.send(DiscordEvent::ChannelDiscovered {
                channel_id: channel.id.get().to_string(),
                display_name: format!("Discord / #{}", channel.name),
            });
        }
    }

    async fn message(&self, _ctx: Context, message: Message) {
        let own_id = self.bot_user_id.lock().ok().and_then(|id| *id);
        if own_id == Some(message.author.id) || message.content.is_empty() {
            return;
        }
        let _ = self.tx.send(DiscordEvent::Message {
            channel_id: message.channel_id.get().to_string(),
            dm_display_name: message
                .guild_id
                .is_none()
                .then(|| format!("DM: {}", message.author.name)),
            sender: message.author.name,
            body: message.content,
        });
    }
}

pub fn bridge_discord_events(
    mut rx: mpsc::UnboundedReceiver<DiscordEvent>,
    sender: ComponentSender<AppModel>,
) {
    runtime::spawn(async move {
        while let Some(event) = rx.recv().await {
            match event {
                DiscordEvent::Connected { user_id } => {
                    sender.input(AppInput::DiscordConnected { user_id });
                }
                DiscordEvent::ChannelDiscovered {
                    channel_id,
                    display_name,
                } => {
                    sender.input(AppInput::DiscordChannelDiscovered {
                        channel_id,
                        display_name,
                    });
                }
                DiscordEvent::Message {
                    channel_id,
                    dm_display_name,
                    sender: message_sender,
                    body,
                } => {
                    sender.input(AppInput::DiscordMessage {
                        channel_id,
                        dm_display_name,
                        sender: message_sender,
                        body,
                    });
                }
                DiscordEvent::Error(error) => sender.input(AppInput::DiscordError(error)),
                DiscordEvent::Disconnected => sender.input(AppInput::DiscordDisconnected),
            }
        }
    });
}
