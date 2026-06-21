# Rawhide Relay

A custom GTK4 IRC client built in Rust using [relm4](https://relm4.org/), themed with Gruvbox Dark.

## Features
- TLS IRC connection (port 6697)
- NickServ authentication
- Multi-channel support with live user lists
- Gruvbox Dark theme via GTK4 CSS

## Dependencies
- Rust + Cargo
- GTK4
- OpenSSL

## Build & Run
```bash
cargo run
```

## Channels
Connects to `irc.libera.chat` with `#fedora-devel`, `#fedora`, and `##rust` by default.

## License
GPL-2.0-or-later
