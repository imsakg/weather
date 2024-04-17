/// Keep untrusted provider/file text from controlling the terminal.
/// JSON is serialized separately and retains the original escaped values.
pub fn terminal(value: &str) -> String {
    value
        .chars()
        .filter(|c| {
            !c.is_control() && !matches!(c, '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}')
        })
        .take(200)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unicode_names_survive_without_terminal_or_bidi_controls() {
        assert_eq!(terminal("İstanbul\u{1b}[2J\n\u{202e}!"), "İstanbul[2J!");
        assert_eq!(terminal(&"é".repeat(300)).chars().count(), 200);
    }
}
