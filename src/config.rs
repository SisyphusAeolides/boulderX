use keyring::Entry;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

const CONFIG_DIR: &str = "boulder-relay";
const CONFIG_FILE: &str = "settings.toml";
const KEYRING_SERVICE: &str = "boulder-relay";

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServerAccount {
    pub nick: String,
    /// Password is NOT serialized to disk — stored in keyring.
    #[serde(skip)]
    pub password: String,
    pub service: String,
    pub auth_method: String, // "nickserv", "sasl_plain", "sasl_external"
}

impl ServerAccount {
    /// Keyring key for this account: "boulder-relay/<server>/<nick>"
    fn keyring_key(server: &str, nick: &str) -> String {
        format!("{}/{}", server, nick)
    }

    /// Load password from keyring. Returns empty string on any error.
    pub fn load_password(server: &str, nick: &str) -> String {
        let key = Self::keyring_key(server, nick);
        Entry::new(KEYRING_SERVICE, &key)
            .and_then(|e| e.get_password())
            .unwrap_or_default()
    }

    /// Save password to keyring. Silently ignores errors (keyring may not be available).
    pub fn save_password(server: &str, nick: &str, password: &str) {
        let key = Self::keyring_key(server, nick);
        if let Ok(entry) = Entry::new(KEYRING_SERVICE, &key) {
            if password.is_empty() {
                let _ = entry.delete_credential();
            } else {
                let _ = entry.set_password(password);
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub nickname: String,
    pub server: String,
    /// Legacy plain-text password field — only used as fallback if keyring unavailable.
    #[serde(default)]
    pub password: String,
    pub favorites: Vec<String>,
    pub extra_channels: Vec<String>,
    pub last_channel: String,
    pub notifications_enabled: bool,
    pub background_on_close: bool,
    pub nick_colors_enabled: bool,
    pub timestamp_format: String,
    pub account_service: String,
    pub auth_method: String,
    pub accounts: HashMap<String, ServerAccount>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            nickname: String::from("SisyphusCode"),
            server: String::from("irc.libera.chat"),
            password: String::new(),
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
        let mut settings: Self = toml::from_str(&content).unwrap_or_default();
        // Reload passwords from keyring for each account
        for (server, acc) in settings.accounts.iter_mut() {
            if acc.password.is_empty() {
                acc.password = ServerAccount::load_password(server, &acc.nick);
            }
        }
        // Also load top-level password from keyring
        if settings.password.is_empty() {
            settings.password =
                ServerAccount::load_password(&settings.server, &settings.nickname);
        }
        settings
    }

    pub fn save(&self) -> std::io::Result<()> {
        let path = config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Save passwords to keyring, strip them from the on-disk struct
        ServerAccount::save_password(&self.server, &self.nickname, &self.password);
        for (server, acc) in &self.accounts {
            ServerAccount::save_password(server, &acc.nick, &acc.password);
        }

        // Build a sanitized copy with passwords zeroed out
        let mut on_disk = self.clone();
        on_disk.password = String::new();
        for acc in on_disk.accounts.values_mut() {
            acc.password = String::new();
        }

        let body = toml::to_string_pretty(&on_disk)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        fs::write(&path, body)?;

        // Restrict permissions to owner-only on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&path, fs::Permissions::from_mode(0o600))?;
        }

        Ok(())
    }
}

fn config_path() -> PathBuf {
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
            ..Settings::default()
        };
        let serialized = toml::to_string_pretty(&settings).unwrap();
        let parsed: Settings = toml::from_str(&serialized).unwrap();
        assert_eq!(parsed.nickname, "testnick");
        assert_eq!(parsed.server, "irc.libera.chat");
    }

    #[test]
    fn default_settings_are_sane() {
        let s = Settings::default();
        assert_eq!(s.nickname, "SisyphusCode");
        assert!(s.notifications_enabled);
        assert!(s.nick_colors_enabled);
        assert_eq!(s.timestamp_format, "%H:%M");
    }

    #[test]
    fn account_keyring_key_format() {
        let key = ServerAccount::keyring_key("irc.libera.chat", "alice");
        assert_eq!(key, "irc.libera.chat/alice");
    }
}
