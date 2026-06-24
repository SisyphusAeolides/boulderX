use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

const CONFIG_DIR: &str = "boulder-relay";
const CONFIG_FILE: &str = "settings.conf";

#[derive(Debug, Clone)]
pub struct Settings {
    pub nickname: String,
    pub server: String,
    pub password: String,
    pub favorites: Vec<String>,
    pub last_channel: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            nickname: String::from("SisyphusCode"),
            server: String::from("irc.libera.chat"),
            password: String::new(),
            favorites: vec![
                String::from("Server"),
                String::from("#rockylinux"),
                String::from("#rockylinux-devel"),
            ],
            last_channel: String::from("#rockylinux-devel"),
        }
    }
}

impl Settings {
    pub fn load() -> Self {
        let path = config_path();
        let Ok(content) = fs::read_to_string(&path) else {
            return Self::default();
        };

        let mut values = parse_key_values(&content);
        let mut settings = Self::default();

        if let Some(nickname) = values.remove("nickname") {
            settings.nickname = nickname;
        }
        if let Some(server) = values.remove("server") {
            settings.server = server;
        }
        if let Some(password) = values.remove("password") {
            settings.password = password;
        }
        if let Some(favorites) = values.remove("favorites") {
            settings.favorites = favorites
                .split('|')
                .filter(|item| !item.is_empty())
                .map(str::to_string)
                .collect();
        }
        if let Some(last_channel) = values.remove("last_channel") {
            settings.last_channel = last_channel;
        }

        settings
    }

    pub fn save(&self) -> std::io::Result<()> {
        let path = config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let favorites = self.favorites.join("|");
        let body = format!(
            "nickname={}\nserver={}\npassword={}\nfavorites={}\nlast_channel={}\n",
            escape_value(&self.nickname),
            escape_value(&self.server),
            escape_value(&self.password),
            escape_value(&favorites),
            escape_value(&self.last_channel),
        );
        fs::write(path, body)
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

fn parse_key_values(content: &str) -> HashMap<String, String> {
    content
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                return None;
            }
            let (key, value) = line.split_once('=')?;
            Some((key.trim().to_string(), unescape_value(value.trim())))
        })
        .collect()
}

fn escape_value(value: &str) -> String {
    value.replace('\\', "\\\\").replace('\n', "\\n")
}

fn unescape_value(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut chars = value.chars();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            match chars.next() {
                Some('\\') => out.push('\\'),
                Some('n') => out.push('\n'),
                Some(other) => {
                    out.push('\\');
                    out.push(other);
                }
                None => out.push('\\'),
            }
        } else {
            out.push(ch);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_escaped_values() {
        let settings = Settings {
            nickname: String::from("test\\nick"),
            server: String::from("irc.libera.chat"),
            password: String::from("sec\\ret"),
            favorites: vec![String::from("#rockylinux")],
            last_channel: String::from("#rockylinux-devel"),
        };
        let encoded = format!(
            "nickname={}\npassword={}\n",
            escape_value(&settings.nickname),
            escape_value(&settings.password),
        );
        let parsed = parse_key_values(&encoded);
        assert_eq!(parsed["nickname"], "test\\nick");
        assert_eq!(parsed["password"], "sec\\ret");
    }
}
