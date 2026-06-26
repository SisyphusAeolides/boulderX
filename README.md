# Boulder Relay

A GTK4 IRC client built in Rust using [relm4](https://relm4.org/), tuned for Fedora, RHEL, and Rocky Linux communities on Libera.Chat.

Named for the Sisyphus myth — the conversation you keep pushing uphill.

## Features

- TLS IRC connection (port 6697)
- NickServ authentication (required for Fedora, RHEL, and Rocky Linux channels)
- Multi-channel support with live user lists
- Channel favorites, grouped sidebar, and per-user mute
- Persistent settings (`~/.config/boulder-relay/settings.conf`)
- Connect / disconnect controls
- Timestamps and auto-scrolling chat view
- Slash commands: `/join`, `/msg`, `/nick`, `/part`, `/clear`, `/help`
- Gruvbox Dark theme with community color accents

## Default channels

On connect, the client joins community channels on `irc.libera.chat`:

| Channel | Community | Purpose |
|---------|-----------|---------|
| `#rockylinux` | Rocky Linux | General support and discussion |
| `#rockylinux-devel` | Rocky Linux | Development and release engineering |
| `#rockylinux-social` | Rocky Linux | Off-topic and social chat |
| `#fedora` | Fedora | General Fedora support and discussion |
| `#fedora-devel` | Fedora | Development, packaging, and infrastructure |
| `#rhel` | RHEL | RHEL support and enterprise Linux discussion |

See the [Rocky Linux IRC wiki](https://wiki.rockylinux.org/irc/) and [Fedora communications docs](https://docs.fedoraproject.org/en-US/project/communications/) for registration and channel details.

## Install from COPR

```bash
sudo dnf copr enable sisyphuscode/boulder-relay
sudo dnf install boulder-relay
```

Builds are provided for **EPEL 9**, **EPEL 10**, **Fedora 44**, and Fedora Rawhide.

Join any Libera.Chat channel from the sidebar: type `#channel` in the join box, use `/join channel` (the `#` is optional), or `/j channel`. Custom channels are remembered between sessions.

On RHEL 10 / Rocky Linux 10 / Alma 10, enable EPEL 10 first if it is not already enabled.

## Development setup

Install build dependencies on Rocky Linux 9 / 10 or Fedora:

```bash
sudo dnf install -y cargo rust gtk4-devel openssl-devel desktop-file-utils libappstream-glib
```

The project pins `relm4 0.8` / `gtk4 0.8` (with default features disabled) so it builds against the GLib and Pango libraries shipped on EL9 and EL10.

Build and run locally:

```bash
cargo run
```

Build an RPM (offline, using vendored crates):

```bash
./packaging/build-rpm.sh
```

Or manually:

```bash
cargo build --release --offline
rpmbuild -ba packaging/boulder-relay.spec
```

Refresh vendored sources after dependency changes:

```bash
cargo vendor vendor
```

## Dependencies

- Rust + Cargo
- GTK4 development libraries (`gtk4-devel`)
- OpenSSL development libraries (`openssl-devel`)

## License

GPL-2.0-or-later