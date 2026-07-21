use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

const CONFIG_DIR: &str = "boulder-relay";
const CONFIG_FILE: &str = "settings.toml";

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServerAccount {
    pub nick: String,
    #[serde(default)]
    pub password: String,
    #[serde(default = "default_service")]
    pub service: String,
    #[serde(default = "default_auth")]
    pub auth_method: String,
}

fn default_service() -> String {
    String::from("NickServ")
}

fn default_auth() -> String {
    String::from("nickserv")
}

impl ServerAccount {
    /// Passwords live in settings.toml (mode 0600). Keyring hooks can replace these later.
    pub fn load_password(server: &str, nick: &str) -> String {
        let settings = Settings::load();
        settings
            .accounts
            .get(server)
            .filter(|a| a.nick == nick || nick.is_empty())
            .map(|a| a.password.clone())
            .unwrap_or_default()
    }

    pub fn save_password(server: &str, nick: &str, password: &str) {
        let mut settings = Settings::load();
        let entry = settings
            .accounts
            .entry(server.to_string())
            .or_insert_with(|| ServerAccount {
                nick: nick.to_string(),
                password: String::new(),
                service: default_service(),
                auth_method: default_auth(),
            });
        if !nick.is_empty() {
            entry.nick = nick.to_string();
        }
        entry.password = password.to_string();
        let _ = settings.save();
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MatrixAccount {
    #[serde(default)]
    pub homeserver: String,
    #[serde(default)]
    pub username: String,
    /// Stored only when the user opts in; file mode is 0600.
    #[serde(default)]
    pub password: String,
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct DiscordAccount {
    /// Bot token only. Stored in settings.toml, whose mode is kept at 0600.
    #[serde(default)]
    pub bot_token: String,
}

impl std::fmt::Debug for DiscordAccount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DiscordAccount")
            .field("bot_token", &"[redacted]")
            .finish()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub nickname: String,
    pub server: String,
    pub password: String,
    /// IRC server port (default 6697 for TLS).
    #[serde(default = "default_irc_port")]
    pub irc_port: u16,
    /// Whether to use TLS for IRC (default true).
    #[serde(default = "default_irc_tls")]
    pub irc_use_tls: bool,
    pub favorites: Vec<String>,
    pub extra_channels: Vec<String>,
    pub last_channel: String,
    pub notifications_enabled: bool,
    pub background_on_close: bool,
    pub nick_colors_enabled: bool,
    pub timestamp_format: String,
    pub account_service: String,
    pub auth_method: String,
    #[serde(default)]
    pub accounts: HashMap<String, ServerAccount>,
    #[serde(default)]
    pub matrix: MatrixAccount,
    #[serde(default)]
    pub discord: DiscordAccount,
}

fn default_irc_port() -> u16 {
    6697
}

fn default_irc_tls() -> bool {
    true
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            nickname: String::new(),
            server: String::from("irc.libera.chat"),
            password: String::new(),
            irc_port: default_irc_port(),
            irc_use_tls: default_irc_tls(),
            favorites: vec![String::from("Server")],
            extra_channels: Vec::new(),
            last_channel: String::from("Server"),
            notifications_enabled: true,
            background_on_close: true,
            nick_colors_enabled: true,
            timestamp_format: "%H:%M".to_string(),
            account_service: String::from("NickServ"),
            auth_method: String::from("nickserv"),
            accounts: HashMap::new(),
            matrix: MatrixAccount::default(),
            discord: DiscordAccount::default(),
        }
    }
}

impl Settings {
    pub fn load() -> Self {
        let path = config_path();
        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => return Self::default(),
        };
        toml::from_str(&content).unwrap_or_default()
    }

    pub fn save(&self) -> std::io::Result<()> {
        let path = config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let body = toml::to_string_pretty(&self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        fs::write(&path, body)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&path, fs::Permissions::from_mode(0o600))?;
        }

        Ok(())
    }
}

/// Public for tests and diagnostics.
pub fn config_path() -> PathBuf {
    let base = std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            std::env::var("HOME")
                .map(|home| PathBuf::from(home).join(".config"))
                .unwrap_or_else(|_| PathBuf::from(".config"))
        });
    base.join(CONFIG_DIR).join(CONFIG_FILE)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_settings_toml() {
        let settings = Settings {
            nickname: String::from("testnick"),
            server: String::from("irc.libera.chat"),
            irc_port: 6667,
            irc_use_tls: false,
            ..Settings::default()
        };
        let serialized = toml::to_string_pretty(&settings).unwrap();
        let parsed: Settings = toml::from_str(&serialized).unwrap();
        assert_eq!(parsed.nickname, "testnick");
        assert_eq!(parsed.server, "irc.libera.chat");
        assert_eq!(parsed.irc_port, 6667);
        assert!(!parsed.irc_use_tls);
    }

    #[test]
    fn default_settings_are_sane() {
        let s = Settings::default();
        assert_eq!(s.nickname, "");
        assert!(s.notifications_enabled);
        assert!(s.nick_colors_enabled);
        assert_eq!(s.timestamp_format, "%H:%M");
        assert_eq!(s.irc_port, 6697);
        assert!(s.irc_use_tls);
    }

    #[test]
    fn config_path_uses_xdg_config_home() {
        // Structural: path ends with boulder-relay/settings.toml
        let p = config_path();
        assert!(
            p.ends_with("boulder-relay/settings.toml")
                || p.ends_with("boulder-relay\\settings.toml")
        );
    }

    #[test]
    fn missing_port_fields_default_on_parse() {
        let toml = r#"
nickname = "n"
server = "irc.example"
password = ""
favorites = []
extra_channels = []
last_channel = "Server"
notifications_enabled = true
background_on_close = true
nick_colors_enabled = true
timestamp_format = "%H:%M"
account_service = "NickServ"
auth_method = "nickserv"
"#;
        let s: Settings = toml::from_str(toml).unwrap();
        assert_eq!(s.irc_port, 6697);
        assert!(s.irc_use_tls);
    }

    #[test]
    fn discord_bot_token_round_trips_without_debug_exposure() {
        let settings = Settings {
            discord: DiscordAccount {
                bot_token: "bot-token".to_string(),
            },
            ..Settings::default()
        };
        let serialized = toml::to_string_pretty(&settings).unwrap();
        let parsed: Settings = toml::from_str(&serialized).unwrap();
        assert_eq!(parsed.discord.bot_token, "bot-token");
        assert!(!format!("{:?}", parsed.discord).contains("bot-token"));
    }
}
