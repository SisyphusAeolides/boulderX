use gtk::prelude::*;
use relm4::gtk;
use adw::prelude::*;
use adw;

const CSS: &str = r#"
/* ============================================================
   Boulder Relay — Element X-inspired theme
   Gruvbox dark base + Sisyphus Blue accents
   ============================================================ */

.boulder-relay {
    background-color: #1d2021;
    color: #ebdbb2;
    font-family: "Inter", "Cantarell", sans-serif;
    font-size: 14px;
}

/* ── Sidebar ──────────────────────────────────────────────── */
.sidebar {
    background-color: #282828;
    border-right: 1px solid #3c3836;
    min-width: 220px;
}

.sidebar-header {
    padding: 12px 12px 6px 12px;
    border-bottom: 1px solid #3c3836;
}

.app-title {
    font-size: 16px;
    font-weight: 700;
    color: #ebdbb2;
    letter-spacing: 0.02em;
}

.sidebar-section-header {
    font-size: 11px;
    font-weight: 700;
    color: #928374;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    padding: 10px 10px 4px 10px;
}

/* ── Room rows ────────────────────────────────────────────── */
.room-row {
    border-radius: 8px;
    margin: 1px 6px;
    padding: 4px 6px;
    transition: background 120ms ease;
}

.room-row:hover {
    background-color: #3c3836;
}

.room-row-active {
    background-color: #504945;
    border-left: 3px solid #458588;
}

.room-name {
    font-size: 14px;
    font-weight: 500;
    color: #ebdbb2;
}

.room-avatar {
    min-width: 32px;
    min-height: 32px;
    border-radius: 50%;
    background-color: #458588;
    color: #1d2021;
    font-weight: 700;
    font-size: 13px;
    padding: 2px;
}

/* ── Protocol badges ────────────────────────────────────────── */
.protocol-badge {
    font-size: 9px;
    font-weight: 700;
    border-radius: 4px;
    padding: 1px 5px;
    letter-spacing: 0.05em;
}

.badge-irc {
    background-color: #d79921;
    color: #1d2021;
}

.badge-matrix {
    background-color: #689d6a;
    color: #1d2021;
}

/* ── Unread badge ───────────────────────────────────────────── */
.unread-badge {
    background-color: #cc241d;
    color: #fbf1c7;
    border-radius: 10px;
    font-size: 11px;
    font-weight: 700;
    min-width: 18px;
    padding: 0 5px;
}

/* ── Chat panel ────────────────────────────────────────────── */
.chat-panel {
    background-color: #1d2021;
}

.channel-header {
    background-color: #282828;
    border-bottom: 1px solid #3c3836;
    padding: 8px 16px;
}

.channel-title {
    font-size: 16px;
    font-weight: 600;
    color: #ebdbb2;
}

.channel-topic {
    font-size: 13px;
    color: #928374;
    font-style: italic;
}

.chat-view {
    background-color: #1d2021;
    color: #ebdbb2;
    font-family: "JetBrains Mono", "Fira Code", monospace;
    font-size: 13.5px;
    line-height: 1.55;
}

/* ── Composer ──────────────────────────────────────────────── */
.composer {
    background-color: #282828;
    border-top: 1px solid #3c3836;
    padding: 8px 12px;
}

.composer-entry {
    background-color: #32302f;
    border: 1px solid #504945;
    border-radius: 20px;
    color: #ebdbb2;
    padding: 6px 14px;
    font-size: 14px;
}

.composer-entry:focus {
    border-color: #458588;
    box-shadow: 0 0 0 2px alpha(#458588, 0.25);
}

.composer-send {
    background-color: #458588;
    color: #1d2021;
    border-radius: 50%;
    min-width: 36px;
    min-height: 36px;
    font-size: 16px;
    font-weight: 700;
    padding: 0;
}

.composer-send:hover {
    background-color: #83a598;
}

/* ── Users panel ───────────────────────────────────────────── */
.users-panel {
    background-color: #282828;
    border-left: 1px solid #3c3836;
    min-width: 160px;
}

.user-btn {
    background: transparent;
    border: none;
    color: #ebdbb2;
    font-size: 13px;
    text-align: left;
    padding: 3px 6px;
    border-radius: 4px;
}

.user-btn:hover {
    background-color: #3c3836;
}

.muted-user {
    opacity: 0.4;
    text-decoration: line-through;
}

.mute-btn {
    background: transparent;
    border: none;
    font-size: 13px;
    padding: 2px 4px;
    border-radius: 4px;
    color: #928374;
}

/* ── Status pills ───────────────────────────────────────────── */
.status-connected {
    color: #b8bb26;
    font-size: 12px;
    font-weight: 600;
}

.status-connecting {
    color: #fabd2f;
    font-size: 12px;
    font-weight: 600;
}

.status-offline {
    color: #928374;
    font-size: 12px;
}

/* ── Buttons ───────────────────────────────────────────────── */
.fav-btn, .part-btn {
    background: transparent;
    border: none;
    color: #928374;
    font-size: 13px;
    padding: 1px 4px;
    border-radius: 4px;
    min-width: 0;
}

.fav-btn:hover { color: #fabd2f; }
.part-btn:hover { color: #fb4934; }

.suggested-action {
    background-color: #458588;
    color: #1d2021;
    border-radius: 6px;
    font-weight: 600;
}

.suggested-action:hover {
    background-color: #83a598;
}

.destructive-action {
    background-color: #cc241d;
    color: #fbf1c7;
    border-radius: 6px;
    font-weight: 600;
}

.destructive-action:hover {
    background-color: #fb4934;
}

.flat {
    background: transparent;
    border: none;
    color: #a89984;
    font-size: 13px;
    border-radius: 6px;
    padding: 4px 8px;
}

.flat:hover {
    background-color: #3c3836;
    color: #ebdbb2;
}

/* ── Dialogs ───────────────────────────────────────────────── */
.dialog-title {
    font-size: 18px;
    font-weight: 700;
    color: #ebdbb2;
    margin-bottom: 8px;
}

/* ── Scrollbars ────────────────────────────────────────────── */
scrollbar slider {
    background-color: #504945;
    border-radius: 4px;
    min-width: 6px;
    min-height: 6px;
}

scrollbar slider:hover {
    background-color: #665c54;
}

scrollbar trough {
    background-color: transparent;
}
"#;

pub fn load_css() {
    let provider = gtk::CssProvider::new();
    provider.load_from_data(CSS);
    if let Some(display) = gtk::gdk::Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}

pub fn build_titlebar() -> adw::HeaderBar {
    let bar = adw::HeaderBar::new();
    bar.set_show_end_title_buttons(true);
    bar.add_css_class("flat");
    bar
}

pub fn attach_window(window: &gtk::Window) {
    window.add_css_class("boulder-relay");
}
