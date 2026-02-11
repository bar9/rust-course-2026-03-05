use std::collections::HashMap;

/// A simple Markdown-to-text processor.
///
/// Handles a subset of Markdown: headings (`#`), bold (`**`), italic (`*`),
/// and links (`[text](url)`).
pub struct MarkdownProcessor;

impl MarkdownProcessor {
    pub fn new() -> Self {
        MarkdownProcessor
    }

    /// Strip all markdown formatting and return plain text.
    ///
    /// - Removes `#` heading prefixes (and the space after them)
    /// - Removes `**` (bold) and `*` (italic) markers
    /// - Converts `[text](url)` links to just `text`
    pub fn to_plain_text(&self, input: &str) -> String {
        let mut result = String::new();
        for line in input.lines() {
            let trimmed = line.trim_start();
            // Strip heading markers
            let line = if trimmed.starts_with('#') {
                trimmed.trim_start_matches('#').trim_start()
            } else {
                trimmed
            };
            // Process inline elements
            let line = Self::strip_links(line);
            let line = line.replace("**", "");
            let line = line.replace('*', "");
            if !result.is_empty() {
                result.push('\n');
            }
            result.push_str(&line);
        }
        result
    }

    /// Extract all links as `(text, url)` pairs from `[text](url)` syntax.
    pub fn extract_links(&self, input: &str) -> Vec<(String, String)> {
        let mut links = Vec::new();
        let mut remaining = input;
        while let Some(open_bracket) = remaining.find('[') {
            let after_bracket = &remaining[open_bracket + 1..];
            if let Some(close_bracket) = after_bracket.find(']') {
                let text = &after_bracket[..close_bracket];
                let after_close = &after_bracket[close_bracket + 1..];
                if after_close.starts_with('(') {
                    if let Some(close_paren) = after_close.find(')') {
                        let url = &after_close[1..close_paren];
                        links.push((text.to_string(), url.to_string()));
                        remaining = &after_close[close_paren + 1..];
                        continue;
                    }
                }
            }
            remaining = &remaining[open_bracket + 1..];
        }
        links
    }

    /// Count headings by level (1–6). A heading line starts with 1–6 `#` characters
    /// followed by a space.
    pub fn count_headings(&self, input: &str) -> HashMap<u8, usize> {
        let mut counts = HashMap::new();
        for line in input.lines() {
            let trimmed = line.trim_start();
            if trimmed.starts_with('#') {
                let hashes = trimmed.chars().take_while(|&c| c == '#').count();
                if hashes >= 1 && hashes <= 6 && trimmed.chars().nth(hashes) == Some(' ') {
                    *counts.entry(hashes as u8).or_insert(0) += 1;
                }
            }
        }
        counts
    }

    /// Convert emphasis markers: `**bold**` becomes `BOLD` (uppercase),
    /// `*italic*` becomes `italic` (lowercase).
    ///
    /// Panics if the input contains unmatched `**` markers.
    pub fn transform_emphasis(&self, input: &str) -> String {
        // First handle bold (**...**)
        let result = Self::transform_bold(input);
        // Then handle italic (*...*)
        Self::transform_italic(&result)
    }

    fn strip_links(input: &str) -> String {
        let mut result = String::new();
        let mut remaining = input;
        while let Some(open_bracket) = remaining.find('[') {
            result.push_str(&remaining[..open_bracket]);
            let after_bracket = &remaining[open_bracket + 1..];
            if let Some(close_bracket) = after_bracket.find(']') {
                let text = &after_bracket[..close_bracket];
                let after_close = &after_bracket[close_bracket + 1..];
                if after_close.starts_with('(') {
                    if let Some(close_paren) = after_close.find(')') {
                        result.push_str(text);
                        remaining = &after_close[close_paren + 1..];
                        continue;
                    }
                }
                // Not a valid link, keep the bracket
                result.push('[');
                remaining = after_bracket;
            } else {
                result.push('[');
                remaining = after_bracket;
            }
        }
        result.push_str(remaining);
        result
    }

    fn transform_bold(input: &str) -> String {
        let mut result = String::new();
        let mut remaining = input;
        loop {
            match remaining.find("**") {
                Some(start) => {
                    result.push_str(&remaining[..start]);
                    let after_open = &remaining[start + 2..];
                    match after_open.find("**") {
                        Some(end) => {
                            let bold_text = &after_open[..end];
                            result.push_str(&bold_text.to_uppercase());
                            remaining = &after_open[end + 2..];
                        }
                        None => {
                            panic!("Unmatched bold markers in input");
                        }
                    }
                }
                None => {
                    result.push_str(remaining);
                    break;
                }
            }
        }
        result
    }

    fn transform_italic(input: &str) -> String {
        let mut result = String::new();
        let mut remaining = input;
        loop {
            match remaining.find('*') {
                Some(start) => {
                    result.push_str(&remaining[..start]);
                    let after_open = &remaining[start + 1..];
                    match after_open.find('*') {
                        Some(end) => {
                            let italic_text = &after_open[..end];
                            result.push_str(&italic_text.to_lowercase());
                            remaining = &after_open[end + 1..];
                        }
                        None => {
                            // Unmatched single asterisk — keep it
                            result.push('*');
                            remaining = after_open;
                        }
                    }
                }
                None => {
                    result.push_str(remaining);
                    break;
                }
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Helper ──────────────────────────────────────────────

    fn processor() -> MarkdownProcessor {
        MarkdownProcessor::new()
    }

    // ── to_plain_text ──────────────────────────────────────

    #[test]
    fn plain_text_strips_headings() {
        let md = "# Title\n## Subtitle\nBody text";
        let result = processor().to_plain_text(md);
        assert_eq!(result, "Title\nSubtitle\nBody text");
    }

    #[test]
    fn plain_text_strips_bold_and_italic() {
        let md = "This is **bold** and *italic* text";
        let result = processor().to_plain_text(md);
        assert_eq!(result, "This is bold and italic text");
    }

    #[test]
    fn plain_text_converts_links_to_text() {
        let md = "Visit [Rust](https://rust-lang.org) for more";
        let result = processor().to_plain_text(md);
        assert_eq!(result, "Visit Rust for more");
    }

    // ── extract_links ──────────────────────────────────────

    #[test]
    fn extract_links_finds_all_links() {
        let md = "See [Rust](https://rust-lang.org) and [Docs](https://doc.rust-lang.org)";
        let links = processor().extract_links(md);
        assert_eq!(links.len(), 2);
        assert_eq!(
            links[0],
            ("Rust".to_string(), "https://rust-lang.org".to_string())
        );
        assert_eq!(
            links[1],
            ("Docs".to_string(), "https://doc.rust-lang.org".to_string())
        );
    }

    #[test]
    fn extract_links_returns_empty_for_no_links() {
        let links = processor().extract_links("Just plain text here");
        assert!(links.is_empty());
    }

    // ── count_headings ─────────────────────────────────────

    #[test]
    fn count_headings_by_level() {
        let md = "# H1\n## H2a\n## H2b\n### H3\n# Another H1";
        let counts = processor().count_headings(md);
        assert_eq!(counts.get(&1), Some(&2));
        assert_eq!(counts.get(&2), Some(&2));
        assert_eq!(counts.get(&3), Some(&1));
        assert_eq!(counts.get(&4), None);
    }

    // ── transform_emphasis ─────────────────────────────────

    #[test]
    fn transform_emphasis_bold_to_uppercase() {
        let result = processor().transform_emphasis("Say **hello** world");
        assert_eq!(result, "Say HELLO world");
    }

    #[test]
    fn transform_emphasis_italic_to_lowercase() {
        let result = processor().transform_emphasis("Say *GOODBYE* world");
        assert_eq!(result, "Say goodbye world");
    }

    // ── should_panic ───────────────────────────────────────

    #[test]
    #[should_panic(expected = "Unmatched bold markers")]
    fn transform_emphasis_panics_on_unmatched_bold() {
        processor().transform_emphasis("This has **unmatched bold");
    }

    // ── Result-returning test ──────────────────────────────

    #[test]
    fn round_trip_plain_text_is_stable() -> Result<(), String> {
        let md = "# Heading\nSome **bold** and [link](url) text";
        let p = processor();
        let first = p.to_plain_text(md);
        let second = p.to_plain_text(&first);
        if first == second {
            Ok(())
        } else {
            Err(format!("Not stable: first={first:?}, second={second:?}"))
        }
    }

    // ── ignored test ───────────────────────────────────────

    #[test]
    #[ignore = "demonstrates #[ignore] — run with cargo test -- --ignored"]
    fn large_document_performance() {
        let big_doc = "# Heading\nSome **bold** and *italic* text\n".repeat(10_000);
        let result = processor().to_plain_text(&big_doc);
        assert!(result.len() > 100_000);
    }
}
