# boulderX

A fast, clean GTK4 + libadwaita IRC/Matrix client written in **100% Rust** using [relm4](https://relm4.org/).

Named for the Sisyphus myth — the conversation you keep pushing uphill.

**v0.6.0** — unified IRC + Matrix support, Gruvbox dark theme, and Sisyphus Blue accents.

---

## Features

- **Dual-protocol**: IRC and Matrix side-by-side in a single unified sidebar with protocol badges
- **X-style UI**: room avatars, unread count pills, rounded composer, bubble chat view
- **Multi-server IRC**: concurrent connections, per-server channels, history, accounts, and state
- **TLS IRC** (port 6697 default), configurable port/plain fallback
- **Modern IRC auth**: NickServ, SASL PLAIN, SASL EXTERNAL (client cert), configurable per server
- **Matrix login**: password or SSO token, E2E encryption via `matrix-sdk`, SQLite store
- **Account management**: Register, Verify, change password, ghost nick — all in-app
- Multi-channel + DM support with native GtkListBox (keyboard nav, hover, selection)
- **Per-nickname coloring** (toggleable, Gruvbox palette)
- Channel topics, per-channel highlights, `/ignore`, mute per user
- Persistent per-server accounts and settings (TOML)
- Auto-reconnect, configurable timestamps, auto-scroll
- Full slash command set: `/join`, `/part`, `/msg`, `/me`, `/nick`, `/whois`, `/away`, `/back`, `/topic`, `/ignore`, `/unignore`, `/clear`, `/list`, `/help`
- **Channel discovery**: sidebar filter + Browse dialog with search, user counts, topics
- **Preferences**: nick colors, timestamp format, auth method
- **Log viewer**: built-in full-text search across history
- Background/tray mode on window close
- Desktop notifications (libnotify)
- GPLv2+ — fully free and open source

---

## Quick Start

1. Click **Accounts** (or **IRC**) and set nick + optional NickServ/SASL password + server.
2. Click **Connect IRC** / **Connect** — TLS to port 6697 by default.
3. Type `#channel` or nick in the join box and press Enter, or use `/join #chan`.
4. Click **MX** (or Accounts → Matrix) to sign in — homeserver + credentials.
5. Use **Register…** / **Verify…** in Accounts for NickServ registration flows.

All joined channels, Matrix rooms, favorites, and saved accounts persist between sessions
(`~/.config/boulder-relay/settings.toml`, mode `0600`).

---

## Install

### Fedora (COPR)

```bash
dnf copr enable sisyphuscode/boulderX
dnf install boulderX
```

### Fedora — From Source

Install build dependencies:

```bash
sudo dnf install rust cargo gtk4-devel libadwaita-devel openssl-devel desktop-file-utils libappstream-glib
```

Build and run:

```bash
cargo build --release
./target/release/boulderX
```

Or just:

```bash
cargo run
```

### RPM — Build locally

```bash
bash packaging/build-rpm.sh
```

This will create an installable `.rpm` in `~/rpmbuild/RPMS/`.

---

## Development

On Fedora:

```bash
sudo dnf install rust cargo gtk4-devel libadwaita-devel openssl-devel
cargo run
```

---

## Module Layout

```
src/
  main.rs          — entry point
  app.rs           — unified AppModel (IRC + Matrix), all AppInput handlers
  irc/
    connection.rs  — IrcConnection::spawn(), full IRC event loop
    commands.rs    — command parsing helpers
  matrix/
    client.rs      — MatrixClient wrapper (login, send, sync)
    rooms.rs       — RoomRegistry (unread counts, room metadata)
    sync.rs        — MatrixEvent → AppInput bridge
  ui/
    sidebar.rs     — Element X room rows, protocol badges, section headers
    chat_view.rs   — bubble renderer, tag setup, history render
    composer.rs    — bottom input bar
    dialogs.rs     — Matrix login + join room dialogs
  channels.rs      — channel name parsing helpers
  config.rs        — TOML settings load/save
  notify.rs        — desktop notifications
  theme.rs         — CSS load, titlebar, window attach
```

---

## Packaging Notes

- Fedora RPM spec in `packaging/boulderX.spec`
- COPR automation in `.copr/Makefile`
- AppStream metainfo in `packaging/org.Sisyphus.BoulderX.metainfo.xml`
- Icons at 128×128 and 256×256 in `assets/`
- Requires: `gtk4`, `libadwaita`, `openssl-libs`, `sqlite-libs`
- Offline vendored build supported (`cargo build --release --offline`)

---

## License

GPL-2.0-or-later
