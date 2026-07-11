# Boulder Relay

A fast, clean GTK4 + libadwaita IRC client written in **100% Rust** using [relm4](https://relm4.org/).

Named for the Sisyphus myth — the conversation you keep pushing uphill.

**The best IRC client for modern distros**: Built for Arch-based systems and any Rust-loving desktop. Supports multiple servers concurrently, modern auth (SASL), per-network accounts, rich UI, and everything you need for daily use without leaving your terminal-less workflow.

Fully generic — no distro defaults.

## Features

- **Multi-server**: Full support for concurrent connections. Add/switch servers in sidebar. Per-server channels, history, accounts, and state.
- TLS (or plain) IRC connections (configurable port)
- **Modern auth**: NickServ, SASL PLAIN, SASL EXTERNAL (client cert). Configurable per server.
- **Account management**: Register, Verify, Change password, Ghost nick, List accounts — all in-app with dialogs.
- Multi-channel + DM support with native GtkListBox (keyboard nav, hover, selection).
- **Per-nickname coloring** in chat and user list (toggleable).
- Channel topics, per-channel highlights and /ignore.
- Persistent per-server accounts and settings.
- Connect / disconnect with auto-reconnect, configurable timestamps, auto-scroll.
- Slash commands + GUI for everything: `/join`, `/list`, `/me`, account ops, ignore, etc.
- **Channel discovery**: Sidebar filter + powerful Browse dialog with search, counts, topics.
- **Preferences & Theming**: Nick colors, timestamps, auth method, theme picker (Gruvbox, Sisyphus Blue, Adwaita). Improved CSS with Sisyphus accents, density, fonts.
- **Logs & Search**: Built-in log viewer with full-text search across history. Auto-saves logs.
- **Rich UI**: libadwaita + Gruvbox, tray/minimize to background support, spellcheck placeholder in input, IRCv3 metadata basics (typing hints via caps).
- Fully generic — the ultimate Rust IRC client for any distro or desktop.

## Quick start

1. Set your nick + optional NickServ password.
2. Set server (default: `irc.libera.chat`).
3. **Connect**.
4. In sidebar: type `#channel` or nick and press Enter, or use `/join #chan`.

All joined channels and favorites persist. No forced defaults on first run — only the Server tab.

## Appearance & Theming

- Fully custom Gruvbox dark theme layered on libadwaita.
- Toggleable nickname colors in Preferences.
- Configurable timestamp format.
- Native ListBox widgets for channels and users.
- Channel topics shown live.
- In-chat search field.
- Future: full light theme variant and more prefs.

The icon is the official Sisyphus logo.

## Multi-Server & Accounts

- Add servers via the "Add server" field.
- Per-server saved nicks, passwords, services, and auth methods.
- Account Manager dialog for change password, ghost, list.
- Full registration + verify flow with email or email-less.

## Logs, Search & Polish

- View Logs button opens searchable history.
- Auto-reconnect, /ignore, per-channel rules.
- Tray + background mode.
- Theme picker in Preferences.

## Install

### Arch Linux

```bash
sudo pacman -S boulder-relay
```

Or from AUR:

```bash
yay -S boulder-relay-git
```

### From source

```bash
cargo run
```

### From source (release build)

```bash
cargo build --release
./target/release/boulder-relay
```

## Development

On Arch Linux:

```bash
sudo pacman -S rust gtk4 libadwaita openssl
cargo run
```

Modern stack. Supports multi-server, SASL, logs, theming. Gruvbox dark theme on Adwaita.

## Packaging notes

- Full feature set packaged: multi-server, SASL, logs, account tools, modern UI.
- Icons at multiple sizes (128x128, 256x256).
- Requires libadwaita and gtk4. Works on any modern Linux distribution.

## License

GPL-2.0-or-later