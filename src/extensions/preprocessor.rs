//! Preprocessor utilities for conflict resolution
//!
//! This module handles early-stage text processing before Markdown parsing.

use once_cell::sync::Lazy;
use regex::Regex;

// Discord-style underline pattern: __text__
static DISCORD_UNDERLINE: Lazy<Regex> = Lazy::new(|| Regex::new(r"__([^_]+)__").unwrap());
static TASKLIST_INDETERMINATE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^([ \t]*(?:[-+*]|\d+\.)\s+)\[-\](\s|$)").unwrap());

const CODEBLOCK_FILENAME_LANGLESS_MARKER: &str = "umd-nolang";
const CODEBLOCK_FILENAME_META_PREFIX: &str = "umd-filename:";

/// Remove comment syntax from input
///
/// Removes single-line comments (`//`) and multi-line comments (`/* ... */`)
/// while preserving comments inside code blocks and inline code.
///
/// # Arguments
///
/// * `input` - The raw markup input
///
/// # Returns
///
/// String with comments removed
pub fn remove_comments(input: &str) -> String {
    let ends_with_newline = input.ends_with('\n');
    let mut result = String::new();
    let mut in_code_block = false;
    let mut code_fence_marker = "";
    let mut in_multiline_comment = false;

    for line in input.lines() {
        // Detect code block start/end
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            if !in_code_block {
                in_code_block = true;
                code_fence_marker = if trimmed.starts_with("```") {
                    "```"
                } else {
                    "~~~"
                };
            } else if trimmed.contains(code_fence_marker) {
                in_code_block = false;
            }
            result.push_str(line);
            result.push('\n');
            continue;
        }

        // Inside code block: preserve everything
        if in_code_block {
            result.push_str(line);
            result.push('\n');
            continue;
        }

        // Process line outside code blocks
        let mut processed_line = String::new();
        let mut chars = line.chars().peekable();
        let mut in_inline_code = false;
        let mut prev_ch = '\0';

        while let Some(ch) = chars.next() {
            // Detect inline code
            if ch == '`' {
                in_inline_code = !in_inline_code;
                processed_line.push(ch);
                prev_ch = ch;
                continue;
            }

            // Inside inline code: preserve everything
            if in_inline_code {
                processed_line.push(ch);
                prev_ch = ch;
                continue;
            }

            // Multi-line comment start: /*
            if !in_multiline_comment && ch == '/' && chars.peek() == Some(&'*') {
                in_multiline_comment = true;
                chars.next(); // consume '*'
                prev_ch = '*';
                continue;
            }

            // Multi-line comment end: */
            if in_multiline_comment && ch == '*' && chars.peek() == Some(&'/') {
                in_multiline_comment = false;
                chars.next(); // consume '/'
                prev_ch = '/';
                continue;
            }

            // Single-line comment start: //
            // But NOT if preceded by ':' (URL scheme like https://)
            if !in_multiline_comment && ch == '/' && chars.peek() == Some(&'/') && prev_ch != ':' {
                // Skip rest of line
                break;
            }

            // Normal character (not in comment)
            if !in_multiline_comment {
                processed_line.push(ch);
                prev_ch = ch;
            }
        }

        // Add processed line if not empty or if we're still in multiline comment
        if !processed_line.trim().is_empty() {
            result.push_str(&processed_line);
            result.push('\n');
        } else if !in_multiline_comment {
            // Preserve empty lines (important for Markdown structure)
            result.push('\n');
        }
    }

    // Remove trailing newline if input didn't have one
    if !ends_with_newline && result.ends_with('\n') {
        result.pop();
    }

    result
}

/// Convert indeterminate task list marker `[-]` to a placeholder.
///
/// The placeholder is later converted to an indeterminate checkbox in HTML.
pub fn preprocess_tasklist_indeterminate(input: &str) -> String {
    let ends_with_newline = input.ends_with('\n');
    let mut result = String::new();
    let mut in_code_block = false;
    let mut code_fence_marker = "";

    for line in input.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            if !in_code_block {
                in_code_block = true;
                code_fence_marker = if trimmed.starts_with("```") {
                    "```"
                } else {
                    "~~~"
                };
            } else if trimmed.contains(code_fence_marker) {
                in_code_block = false;
            }
            result.push_str(line);
            result.push('\n');
            continue;
        }

        if in_code_block {
            result.push_str(line);
            result.push('\n');
            continue;
        }

        let processed = TASKLIST_INDETERMINATE.replace(line, "$1[ ]{{TASK_INDETERMINATE}}$2");
        result.push_str(&processed);
        result.push('\n');
    }

    if !ends_with_newline && result.ends_with('\n') {
        result.pop();
    }

    result
}

/// Normalize fenced code block info string for filename syntax.
///
/// Converts `lang:filename` to `lang umd-filename:filename` so comrak can emit
/// `data-meta` when `render.full_info_string = true`.
///
/// Also supports `:filename` by using an internal language marker (`umd-nolang`).
pub fn preprocess_code_block_filenames(input: &str) -> String {
    let ends_with_newline = input.ends_with('\n');
    let mut result = String::new();
    let mut in_code_block = false;
    let mut fence_char = '\0';
    let mut fence_len = 0usize;

    for line in input.lines() {
        let trimmed = line.trim_start();

        if !in_code_block {
            if let Some((prefix_len, current_fence_char, current_fence_len, info)) =
                parse_fence_open_line(trimmed)
            {
                in_code_block = true;
                fence_char = current_fence_char;
                fence_len = current_fence_len;

                let normalized_info = normalize_code_fence_info(info);
                let prefix = &line[..line.len() - trimmed.len()];
                let fence_marker = &trimmed[..prefix_len];

                result.push_str(prefix);
                result.push_str(fence_marker);
                if !normalized_info.is_empty() {
                    result.push(' ');
                    result.push_str(&normalized_info);
                }
                result.push('\n');
                continue;
            }
        } else if is_fence_close_line(trimmed, fence_char, fence_len) {
            in_code_block = false;
            fence_char = '\0';
            fence_len = 0;
            result.push_str(line);
            result.push('\n');
            continue;
        }

        result.push_str(line);
        result.push('\n');
    }

    if !ends_with_newline && result.ends_with('\n') {
        result.pop();
    }

    result
}

fn parse_fence_open_line(trimmed_line: &str) -> Option<(usize, char, usize, &str)> {
    let bytes = trimmed_line.as_bytes();
    if bytes.is_empty() {
        return None;
    }

    let first = bytes[0] as char;
    if first != '`' && first != '~' {
        return None;
    }

    let mut marker_len = 0usize;
    while marker_len < bytes.len() && (bytes[marker_len] as char) == first {
        marker_len += 1;
    }

    if marker_len < 3 {
        return None;
    }

    let info = trimmed_line[marker_len..].trim();
    Some((marker_len, first, marker_len, info))
}

fn is_fence_close_line(trimmed_line: &str, fence_char: char, fence_len: usize) -> bool {
    if trimmed_line.is_empty() || fence_len < 3 {
        return false;
    }

    let marker: String = std::iter::repeat(fence_char).take(fence_len).collect();
    if !trimmed_line.starts_with(&marker) {
        return false;
    }

    trimmed_line[fence_len..].trim().is_empty()
}

fn normalize_code_fence_info(info: &str) -> String {
    if info.is_empty() || info.contains(' ') {
        return info.to_string();
    }

    if let Some(filename) = info.strip_prefix(':') {
        if filename.is_empty() {
            return info.to_string();
        }
        return format!(
            "{} {}{}",
            CODEBLOCK_FILENAME_LANGLESS_MARKER, CODEBLOCK_FILENAME_META_PREFIX, filename
        );
    }

    if let Some((lang, filename)) = info.split_once(':') {
        if lang.is_empty() || filename.is_empty() {
            return info.to_string();
        }
        return format!("{} {}{}", lang, CODEBLOCK_FILENAME_META_PREFIX, filename);
    }

    info.to_string()
}

/// Process definition lists (:term|definition syntax)
///
/// Converts consecutive lines starting with `:term|definition` into
/// marker placeholders that will be converted to HTML later.
pub fn process_definition_lists(input: &str) -> String {
    let mut result = Vec::new();
    let mut lines = input.lines().peekable();

    while let Some(line) = lines.next() {
        // Check if this line starts a definition list
        if line.trim_start().starts_with(':') && line.contains('|') {
            let mut dl_items = Vec::new();

            // Collect consecutive definition list items
            let mut current_line = line;
            loop {
                if let Some(stripped) = current_line.trim_start().strip_prefix(':') {
                    if let Some((term, definition)) = stripped.split_once('|') {
                        dl_items.push((term.trim().to_string(), definition.trim().to_string()));
                    }
                }

                // Check if next line is also a definition list item
                match lines.peek() {
                    Some(next_line)
                        if next_line.trim_start().starts_with(':') && next_line.contains('|') =>
                    {
                        current_line = lines.next().unwrap();
                    }
                    _ => break,
                }
            }

            // Create marker for the definition list
            if !dl_items.is_empty() {
                let items_json = serde_json::to_string(&dl_items).unwrap();
                result.push(format!(
                    "{{{{DEFINITION_LIST:{}:DEFINITION_LIST}}}}",
                    items_json
                ));
            }
        } else {
            result.push(line.to_string());
        }
    }

    result.join("\n")
}

/// Convert Discord-style underline (__text__) to placeholder before Markdown parsing
///
/// This prevents CommonMark from converting __text__ to <strong>
pub fn preprocess_discord_underline(input: &str) -> String {
    DISCORD_UNDERLINE
        .replace_all(input, "{{UNDERLINE:$1:UNDERLINE}}")
        .to_string()
}

/// Restore Discord-style underline placeholders to <u> tags
///
/// This should be called after Markdown parsing
pub fn postprocess_discord_underline(html: &str) -> String {
    html.replace("{{UNDERLINE:", "<u>")
        .replace(":UNDERLINE}}", "</u>")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_single_line_comment() {
        let input = "text // comment\nmore text";
        let output = remove_comments(input);
        assert!(!output.contains("comment"));
        assert!(output.contains("text"));
        assert!(output.contains("more text"));
    }

    #[test]
    fn test_remove_multiline_comment() {
        let input = "text /* comment */ more";
        let output = remove_comments(input);
        assert!(!output.contains("comment"));
        assert!(output.contains("text"));
        assert!(output.contains("more"));
    }

    #[test]
    fn test_preserve_url_slashes() {
        let input = "https://example.com";
        let output = remove_comments(input);
        assert!(output.contains("https://example.com"));
    }

    #[test]
    fn test_preserve_comments_in_code_block() {
        let input = "```\n// code comment\n```";
        let output = remove_comments(input);
        assert!(output.contains("// code comment"));
    }

    #[test]
    fn test_definition_list() {
        let input = ":term1|definition1\n:term2|definition2\nregular text";
        let output = process_definition_lists(input);
        assert!(output.contains("{{DEFINITION_LIST:"));
        assert!(output.contains("DEFINITION_LIST}}"));
        assert!(output.contains("regular text"));
    }

    #[test]
    fn test_tasklist_indeterminate() {
        let input = "- [-] Maybe";
        let output = preprocess_tasklist_indeterminate(input);
        assert!(output.contains("- [ ]{{TASK_INDETERMINATE}} Maybe"));
    }

    #[test]
    fn test_tasklist_indeterminate_ignores_code_block() {
        let input = "```\n- [-] Maybe\n```";
        let output = preprocess_tasklist_indeterminate(input);
        assert!(output.contains("- [-] Maybe"));
    }

    #[test]
    fn test_preprocess_discord_underline() {
        let input = "This is __underlined__ text.";
        let output = preprocess_discord_underline(input);
        assert!(output.contains("{{UNDERLINE:underlined:UNDERLINE}}"));
        assert!(!output.contains("__underlined__"));
    }

    #[test]
    fn test_postprocess_discord_underline() {
        let input = "<p>This is {{UNDERLINE:underlined:UNDERLINE}} text.</p>";
        let output = postprocess_discord_underline(input);
        assert_eq!(output, "<p>This is <u>underlined</u> text.</p>");
    }

    #[test]
    fn test_discord_underline_roundtrip() {
        let input = "Text with __underline__ here.";
        let preprocessed = preprocess_discord_underline(input);
        let html = format!(
            "<p>{}</p>",
            preprocessed.replace("__underline__", "{{UNDERLINE:underline:UNDERLINE}}")
        );
        let output = postprocess_discord_underline(&html);
        assert!(output.contains("<u>underline</u>"));
    }

    #[test]
    fn test_preprocess_code_block_filename_with_language() {
        let input = "```rust:src/main.rs\nfn main() {}\n```";
        let output = preprocess_code_block_filenames(input);
        assert!(output.contains("``` rust umd-filename:src/main.rs"));
    }

    #[test]
    fn test_preprocess_code_block_filename_without_language() {
        let input = "```:config.yml\nkey: value\n```";
        let output = preprocess_code_block_filenames(input);
        assert!(output.contains("``` umd-nolang umd-filename:config.yml"));
    }

    #[test]
    fn test_preprocess_code_block_filename_ignores_inside_block() {
        let input = "```txt\nrust:main.rs\n```";
        let output = preprocess_code_block_filenames(input);
        assert!(output.contains("rust:main.rs"));
    }
}
