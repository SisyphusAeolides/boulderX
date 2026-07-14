/// Parse a raw slash command string into a target and body for /msg.
pub fn parse_msg_command(input: &str) -> Option<(String, String)> {
    let mut parts = input.trim_start_matches("/msg ").splitn(2, ' ');
    let target = parts.next()?.to_string();
    let body = parts.next()?.to_string();
    if target.is_empty() || body.is_empty() { return None; }
    Some((target, body))
}
