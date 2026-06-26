#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Community {
    Rocky,
    Fedora,
}

pub struct ChannelDef {
    pub name: &'static str,
    pub description: &'static str,
    pub community: Community,
}

/// Default Libera.Chat channels for Rocky Linux, Fedora, and RHEL communities.
pub const DEFAULT_CHANNELS: &[ChannelDef] = &[
    ChannelDef {
        name: "#rockylinux",
        description: "General Rocky Linux support and discussion",
        community: Community::Rocky,
    },
    ChannelDef {
        name: "#rockylinux-devel",
        description: "Rocky Linux development and release engineering",
        community: Community::Rocky,
    },
    ChannelDef {
        name: "#rockylinux-social",
        description: "Off-topic and social chat for the Rocky community",
        community: Community::Rocky,
    },
    ChannelDef {
        name: "#fedora",
        description: "General Fedora support and discussion",
        community: Community::Fedora,
    },
    ChannelDef {
        name: "#fedora-devel",
        description: "Fedora development, packaging, and infrastructure",
        community: Community::Fedora,
    },
    ChannelDef {
        name: "#rhel",
        description: "RHEL support and enterprise Linux discussion",
        community: Community::Fedora,
    },
];

pub const COMMUNITY_ORDER: &[Community] = &[Community::Rocky, Community::Fedora];

pub fn default_channel_names() -> Vec<String> {
    DEFAULT_CHANNELS
        .iter()
        .map(|channel| channel.name.to_string())
        .collect()
}

pub fn channel_info(name: &str) -> Option<&'static ChannelDef> {
    DEFAULT_CHANNELS.iter().find(|channel| channel.name == name)
}

pub fn community_for(name: &str) -> Option<Community> {
    channel_info(name).map(|channel| channel.community)
}

pub fn community_label(community: Community) -> &'static str {
    match community {
        Community::Rocky => "Rocky Linux",
        Community::Fedora => "Fedora & RHEL",
    }
}

pub fn community_css_class(community: Community) -> &'static str {
    match community {
        Community::Rocky => "channel-rocky",
        Community::Fedora => "channel-fedora",
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JoinTarget {
    Channel(String),
    DirectMessage(String),
}

pub fn is_channel_target(name: &str) -> bool {
    let name = name.trim();
    name.starts_with('#')
        || name.starts_with('&')
        || name.starts_with('+')
        || name.starts_with('!')
}

pub fn normalize_channel_name(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    if is_channel_target(trimmed) {
        trimmed.to_string()
    } else {
        format!("#{trimmed}")
    }
}

/// Parse a `/join` argument (always treated as a channel).
pub fn parse_join_command(raw: &str) -> Option<String> {
    let raw = raw.trim();
    if raw.is_empty() {
        None
    } else {
        Some(normalize_channel_name(raw))
    }
}

/// Parse the sidebar join entry: `#channel` joins a channel, plain text opens a DM.
pub fn parse_join_entry(raw: &str) -> Option<JoinTarget> {
    let raw = raw.trim();
    if raw.is_empty() {
        return None;
    }

    if is_channel_target(raw) {
        Some(JoinTarget::Channel(raw.to_string()))
    } else {
        Some(JoinTarget::DirectMessage(raw.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn includes_requested_channels() {
        let names: Vec<_> = default_channel_names();
        for expected in ["#fedora", "#fedora-devel", "#rhel"] {
            assert!(names.contains(&expected.to_string()), "missing {expected}");
        }
    }

    #[test]
    fn normalizes_join_command_targets() {
        assert_eq!(parse_join_command("fedora-devel"), Some("#fedora-devel".into()));
        assert_eq!(parse_join_command("#fedora"), Some("#fedora".into()));
        assert_eq!(parse_join_command("##unofficial"), Some("##unofficial".into()));
    }

    #[test]
    fn parses_join_entry_channels_and_dms() {
        assert_eq!(
            parse_join_entry("#archlinux"),
            Some(JoinTarget::Channel("#archlinux".into()))
        );
        assert_eq!(
            parse_join_entry("alice"),
            Some(JoinTarget::DirectMessage("alice".into()))
        );
    }
}