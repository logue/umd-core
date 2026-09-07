//! `::: 記法` colon-fence notation for block-type plugins.
//!
//! `:::function args` / content / `:::` is, strictly speaking, fence
//! notation (like the ``` code fence in [`super::code_block`]) whose body
//! is dispatched to a block-type plugin — so the fence *parsing* (finding
//! the opening/closing lines, capturing the raw content) lives here, while
//! the plugin *rendering* (deciding what `math`/`popover`/`clear`/generic
//! function names produce) stays owned by
//! [`crate::extensions::plugins::block`], which this module calls into.

use base64::{Engine as _, engine::general_purpose};
use once_cell::sync::Lazy;
use regex::{Captures, Regex};

use crate::extensions::plugins::block::render_fence_block_plugin;

/// Protect block-type plugin syntax written with the `::: 記法` fence notation
///
/// Converts:
///
/// ```text
/// :::function args
/// content
/// :::
/// ```
///
/// into a safe marker carrying the function name, raw args (the remainder
/// of the opening line), and base64-encoded content.
///
/// A block is closed by the *first* line that consists of exactly `:::`
/// (optionally trailing whitespace). Nesting is not supported: any `:::`
/// block, or `@`/`&` plugin syntax, found between the opening and closing
/// line is captured verbatim as opaque literal content rather than being
/// parsed as a nested plugin.
pub fn protect_fence_block_plugins(input: &str) -> String {
    static START_LINE: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"(?m)^:::(\w+)(?:[ \t]+([^\r\n]*))?[ \t]*\r?\n").unwrap());
    static CLOSE_LINE: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"(?m)^:::[ \t]*(?:\r?\n|$)").unwrap());

    let mut result = String::with_capacity(input.len());
    let mut pos = 0usize;

    while let Some(start_caps) = START_LINE.captures_at(input, pos) {
        let start_match = start_caps.get(0).unwrap();

        // Preserve any text before the opening marker unchanged
        result.push_str(&input[pos..start_match.start()]);

        let function = start_caps.get(1).map_or("", |m| m.as_str());
        let args = start_caps.get(2).map_or("", |m| m.as_str()).trim();
        let content_start = start_match.end();

        match CLOSE_LINE.captures_at(input, content_start) {
            Some(close_caps) => {
                let close_match = close_caps.get(0).unwrap();
                let raw_content = &input[content_start..close_match.start()];
                // The newline terminating the last content line belongs to
                // the closing fence, not the content itself.
                let content = raw_content
                    .strip_suffix("\r\n")
                    .or_else(|| raw_content.strip_suffix('\n'))
                    .unwrap_or(raw_content);
                let encoded_content = general_purpose::STANDARD.encode(content.as_bytes());
                result.push_str(&format!(
                    "{{{{FENCE_BLOCK_PLUGIN:{}:{}:{}:FENCE_BLOCK_PLUGIN}}}}",
                    function, args, encoded_content
                ));
                pos = close_match.end();
            }
            None => {
                // No closing marker found; leave the opening line untouched
                // and keep scanning for other blocks after it.
                result.push_str(&input[start_match.start()..start_match.end()]);
                pos = start_match.end();
            }
        }
    }

    result.push_str(&input[pos..]);
    result
}

/// Restore `{{FENCE_BLOCK_PLUGIN...}}` markers left by
/// `protect_fence_block_plugins`.
///
/// Called from `conflict_resolver::postprocess_conflicts_with_options`
/// alongside `plugins::block::restore_markers` — see that function for the
/// surrounding restore order. The actual per-function rendering (deciding
/// what `math`/`popover`/`clear`/generic function names produce) is
/// delegated to `plugins::block::render_fence_block_plugin`.
pub(crate) fn restore_markers(html: &str) -> String {
    static FENCE_BLOCK_PLUGIN_MARKER: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"\{\{FENCE_BLOCK_PLUGIN:(\w+):([\s\S]*?):([\s\S]*?):FENCE_BLOCK_PLUGIN\}\}")
            .unwrap()
    });

    FENCE_BLOCK_PLUGIN_MARKER
        .replace_all(html, |caps: &Captures| {
            let function = &caps[1];
            let args = &caps[2];
            let encoded_content = &caps[3];

            // Decode base64 to get original content
            let content = general_purpose::STANDARD
                .decode(encoded_content.as_bytes())
                .ok()
                .and_then(|bytes| String::from_utf8(bytes).ok())
                .unwrap_or_else(|| encoded_content.to_string());

            render_fence_block_plugin(function, args, &content)
        })
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protect_fence_block_plugin_basic() {
        let input = ":::alert warning\nSomething went wrong\n:::";
        let output = protect_fence_block_plugins(input);
        assert!(output.contains("FENCE_BLOCK_PLUGIN:alert:warning:"));
        assert!(!output.contains(":::"));
    }

    #[test]
    fn test_protect_fence_block_plugin_no_args() {
        let input = ":::toc\nignored\n:::";
        let output = protect_fence_block_plugins(input);
        assert!(output.contains("FENCE_BLOCK_PLUGIN:toc::"));
    }

    #[test]
    fn test_protect_fence_block_plugin_empty_content() {
        let input = ":::clear\n:::";
        let output = protect_fence_block_plugins(input);
        assert!(output.contains("FENCE_BLOCK_PLUGIN:clear::"));
        assert!(output.contains("FENCE_BLOCK_PLUGIN}}"));
    }

    #[test]
    fn test_protect_fence_block_plugin_multiline_content() {
        let input = ":::box\nline one\nline two\n:::";
        let output = protect_fence_block_plugins(input);

        // Decode the base64 content back out and verify roundtrip
        let marker_re = Regex::new(r"FENCE_BLOCK_PLUGIN:box::([^:]+):FENCE_BLOCK_PLUGIN").unwrap();
        let caps = marker_re
            .captures(&output)
            .expect("marker should be present");
        let decoded = general_purpose::STANDARD.decode(&caps[1]).unwrap();
        assert_eq!(String::from_utf8(decoded).unwrap(), "line one\nline two");
    }

    #[test]
    fn test_protect_fence_block_plugin_unclosed_left_untouched() {
        let input = ":::alert warning\nSomething went wrong";
        let output = protect_fence_block_plugins(input);
        // No closing marker present, so the input is left unchanged
        assert_eq!(output, input);
    }

    #[test]
    fn test_protect_fence_block_plugin_nested_not_supported() {
        // The outer block closes at the FIRST bare `:::` line, which
        // belongs to the inner (nested) block. The nested start marker
        // is therefore captured as opaque literal content, and the
        // outer's real closing `:::` is left over as plain text.
        let input = ":::outer args\n:::inner args2\ncontent\n:::\n:::";
        let output = protect_fence_block_plugins(input);
        assert!(output.contains("FENCE_BLOCK_PLUGIN:outer:args:"));
        // The leftover closing line for the (never-matched) outer block remains
        assert!(output.trim_end().ends_with(":::"));
    }

    #[test]
    fn test_restore_clear_from_fence_block() {
        let protected = protect_fence_block_plugins(":::clear\n:::");
        let output = restore_markers(&protected);
        assert_eq!(output, "<div class=\"clearfix\"></div>");
    }
}
