use gtk::prelude::*;
use relm4::gtk;

pub const NICK_COLORS: [&str; 8] = [
    "#fabd2f", "#b8bb26", "#83a598", "#d3869b",
    "#fe8019", "#8ec07c", "#fb4934", "#d79921",
];

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum LineStyle {
    Normal,
    SelfMsg,
    System,
    Mention,
}

pub fn nick_color_index(nick: &str) -> usize {
    let clean = nick.trim_start_matches(['@', '+', '%', '~', '&']);
    let hash = clean.bytes().fold(0u32, |h, b| h.wrapping_mul(31).wrapping_add(b as u32));
    (hash as usize) % NICK_COLORS.len()
}

pub fn setup_tags(view: &gtk::TextView) {
    let buffer = view.buffer();
    let table = buffer.tag_table();
    for (name, fg, bg) in [
        ("normal", "#ebdbb2", None),
        ("self-msg", "#10B981", None),
        ("system", "#928374", None),
        ("mention", "#fe8019", Some("#3c3836")),
        ("ts", "#665c54", None),
        ("nick", "#83a598", None),
    ] {
        let tag = gtk::TextTag::new(Some(name));
        tag.set_foreground(Some(fg));
        if let Some(b) = bg { tag.set_background(Some(b)); }
        table.add(&tag);
    }
    for (i, &color) in NICK_COLORS.iter().enumerate() {
        let tag_name = format!("nick-{}", i);
        let tag = gtk::TextTag::new(Some(&tag_name));
        tag.set_foreground(Some(color));
        tag.set_weight(600);
        table.add(&tag);
    }
}

pub fn append_bubble(
    view: &gtk::TextView,
    timestamp: &str,
    user: &str,
    body: &str,
    style: LineStyle,
    nick_colors_enabled: bool,
) {
    let buffer = view.buffer();
    let mut end = buffer.end_iter();
    let style_tag = match style {
        LineStyle::Normal => "normal",
        LineStyle::SelfMsg => "self-msg",
        LineStyle::System => "system",
        LineStyle::Mention => "mention",
    };
    buffer.insert_with_tags_by_name(&mut end, timestamp, &["ts"]);
    let nick_part = format!(" {} ", user);
    if user != "System" && nick_colors_enabled {
        let nick_tag = format!("nick-{}", nick_color_index(user));
        buffer.insert_with_tags_by_name(&mut end, &nick_part, &[nick_tag.as_str()]);
    } else {
        buffer.insert_with_tags_by_name(&mut end, &nick_part, &["system"]);
    }
    buffer.insert_with_tags_by_name(&mut end, &format!("{}\n", body), &[style_tag]);
    let mark = buffer.create_mark(None, &buffer.end_iter(), false);
    view.scroll_to_mark(&mark, 0.0, false, 0.0, 0.0);
}

pub fn render_history(
    view: &gtk::TextView,
    lines: &[(String, String, String, LineStyle)],
    nick_colors_enabled: bool,
) {
    view.buffer().set_text("");
    for (ts, user, body, style) in lines {
        append_bubble(view, ts, user, body, *style, nick_colors_enabled);
    }
}
