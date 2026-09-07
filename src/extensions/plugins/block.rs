//! Block plugin notation: `@function(args){{content}}` / `@function(args){content}`
//! / `@function(args)` and the `::: 記法` colon-fence variant.
//!
//! Like `plugins::inline`, this module owns both the "protect" step
//! (`protect_block_plugins` / `protect_colon_block_plugins`, run before
//! Markdown parsing) and the "restore" step (`restore_markers`, run during
//! `conflict_resolver::postprocess_conflicts_with_options`) for block-level
//! plugin syntax. Recognized ("standard") function names (`math`, `popover`,
//! `clear`) get real HTML; anything else falls back to a generic
//! `<template class="umd-plugin-*">`.

use base64::{Engine as _, engine::general_purpose};
use once_cell::sync::Lazy;
use regex::{Captures, Regex};

use super::{escape_html_text, render_args_as_data, render_math_html, render_popover_html};

/// Protect block plugin syntax by converting to markers
///
/// Converts various block plugin patterns into safe markers:
/// - `@function(args){{ content }}` → marker with content
/// - `@function(args){content}` → marker with content
/// - `@function(args)` → marker with args
pub fn protect_block_plugins(input: &str) -> String {
    let mut result = input.to_string();

    // Protect block plugins multiline: @function(args){{ content }}
    let block_plugin_multi = Regex::new(r"@(\w+)\(([^)]*)\)\{\{([\s\S]*?)\}\}").unwrap();
    result = block_plugin_multi
        .replace_all(&result, |caps: &regex::Captures| {
            let function = &caps[1];
            let args = &caps[2];
            let content = &caps[3];
            let encoded_content = general_purpose::STANDARD.encode(content.as_bytes());
            format!(
                "{{{{BLOCK_PLUGIN:{}:{}:{}:BLOCK_PLUGIN}}}}",
                function, args, encoded_content
            )
        })
        .to_string();

    // Protect block plugins singleline: @function(args){content}
    let block_plugin_single = Regex::new(r"@(\w+)\(([^)]*)\)\{([^}]*)\}").unwrap();
    result = block_plugin_single
        .replace_all(&result, |caps: &regex::Captures| {
            let function = &caps[1];
            let args = &caps[2];
            let content = &caps[3];
            let encoded_content = general_purpose::STANDARD.encode(content.as_bytes());
            format!(
                "{{{{BLOCK_PLUGIN:{}:{}:{}:BLOCK_PLUGIN}}}}",
                function, args, encoded_content
            )
        })
        .to_string();

    // Protect block plugins (args only, no content): @function(args)
    let block_plugin_argsonly = Regex::new(r"@(\w+)\(([^)]*)\)").unwrap();
    result = block_plugin_argsonly
        .replace_all(&result, |caps: &regex::Captures| {
            let function = &caps[1];
            let args = &caps[2];
            let encoded_args = general_purpose::STANDARD.encode(args.as_bytes());
            format!(
                "{{{{BLOCK_PLUGIN_ARGSONLY:{}:{}:BLOCK_PLUGIN_ARGSONLY}}}}",
                function, encoded_args
            )
        })
        .to_string();

    result
}

/// Protect block-type plugin syntax written with the `::: 記法` notation
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
pub fn protect_colon_block_plugins(input: &str) -> String {
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
                    "{{{{COLON_BLOCK_PLUGIN:{}:{}:{}:COLON_BLOCK_PLUGIN}}}}",
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

/// Restore `{{BLOCK_PLUGIN...}}`/`{{COLON_BLOCK_PLUGIN...}}` markers left by
/// `protect_block_plugins`/`protect_colon_block_plugins`.
///
/// Called from `conflict_resolver::postprocess_conflicts_with_options` at
/// the same point the three marker-restoration regex blocks used to run
/// inline — see that function for the surrounding restore order.
pub(crate) fn restore_markers(html: &str) -> String {
    let mut result = html.to_string();

    // Restore block plugins
    let block_plugin_marker =
        Regex::new(r"\{\{BLOCK_PLUGIN:(\w+):([\s\S]*?):([\s\S]*?):BLOCK_PLUGIN\}\}").unwrap();
    result = block_plugin_marker
        .replace_all(&result, |caps: &Captures| {
            let function = &caps[1];
            let args = &caps[2];
            let encoded_content = &caps[3];

            // Decode base64 to get original content
            let content = general_purpose::STANDARD
                .decode(encoded_content.as_bytes())
                .ok()
                .and_then(|bytes| String::from_utf8(bytes).ok())
                .unwrap_or_else(|| encoded_content.to_string());

            if function == "math" {
                let formula = if content.trim().is_empty() {
                    args
                } else {
                    &content
                };
                if let Some(mathml) = render_math_html(formula, true) {
                    return mathml;
                }
            }

            if function == "popover" {
                return render_popover_html(args, &content);
            }

            let args_html = render_args_as_data(args);
            let escaped_content = escape_html_text(&content);

            if escaped_content.is_empty() {
                format!(
                    "<template class=\"umd-plugin umd-plugin-{}\">{}</template>",
                    function, args_html
                )
            } else {
                format!(
                    "<template class=\"umd-plugin umd-plugin-{}\">{}{}</template>",
                    function, args_html, escaped_content
                )
            }
        })
        .to_string();

    // Restore colon block plugins (`::: 記法`)
    let colon_block_plugin_marker =
        Regex::new(r"\{\{COLON_BLOCK_PLUGIN:(\w+):([\s\S]*?):([\s\S]*?):COLON_BLOCK_PLUGIN\}\}")
            .unwrap();
    result = colon_block_plugin_marker
        .replace_all(&result, |caps: &Captures| {
            let function = &caps[1];
            let args = &caps[2];
            let encoded_content = &caps[3];

            // Decode base64 to get original content
            let content = general_purpose::STANDARD
                .decode(encoded_content.as_bytes())
                .ok()
                .and_then(|bytes| String::from_utf8(bytes).ok())
                .unwrap_or_else(|| encoded_content.to_string());

            if function == "clear" && args.trim().is_empty() && content.trim().is_empty() {
                return "<div class=\"clearfix\"></div>".to_string();
            }

            if function == "math" {
                let formula = if content.trim().is_empty() {
                    args
                } else {
                    &content
                };
                if let Some(mathml) = render_math_html(formula, true) {
                    return mathml;
                }
            }

            if function == "popover" {
                return render_popover_html(args, &content);
            }

            let args_html = render_args_as_data(args);
            let escaped_content = escape_html_text(&content);

            if escaped_content.is_empty() {
                format!(
                    "<template class=\"umd-plugin umd-plugin-{}\">{}</template>",
                    function, args_html
                )
            } else {
                format!(
                    "<template class=\"umd-plugin umd-plugin-{}\">{}{}</template>",
                    function, args_html, escaped_content
                )
            }
        })
        .to_string();

    // Restore block plugins (args only, no content)
    let block_plugin_argsonly_marker =
        Regex::new(r"\{\{BLOCK_PLUGIN_ARGSONLY:(\w+):([\s\S]*?):BLOCK_PLUGIN_ARGSONLY\}\}")
            .unwrap();
    result = block_plugin_argsonly_marker
        .replace_all(&result, |caps: &Captures| {
            let function = &caps[1];
            let encoded_args = &caps[2];

            // Decode base64 to get original args
            let args = general_purpose::STANDARD
                .decode(encoded_args.as_bytes())
                .ok()
                .and_then(|bytes| String::from_utf8(bytes).ok())
                .unwrap_or_else(|| encoded_args.to_string());

            if function == "clear" && args.trim().is_empty() {
                return "<div class=\"clearfix\"></div>".to_string();
            }

            if function == "math" {
                if let Some(mathml) = render_math_html(&args, true) {
                    return mathml;
                }
            }

            let args_html = render_args_as_data(&args);
            format!(
                "<template class=\"umd-plugin umd-plugin-{}\">{}</template>",
                function, args_html
            )
        })
        .to_string();

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protect_block_plugin_multiline() {
        let input = "@test(args){{ content }}";
        let output = protect_block_plugins(input);
        assert!(output.contains("BLOCK_PLUGIN:test:args:"));
    }

    #[test]
    fn test_protect_block_plugin_single_line() {
        let input = "@test(args){content}";
        let output = protect_block_plugins(input);
        assert!(output.contains("BLOCK_PLUGIN:test:args:"));
    }

    #[test]
    fn test_protect_block_plugin_args_only() {
        let input = "@test(args)";
        let output = protect_block_plugins(input);
        assert!(output.contains("BLOCK_PLUGIN_ARGSONLY:test:"));
    }

    #[test]
    fn test_protect_colon_block_plugin_basic() {
        let input = ":::alert warning\nSomething went wrong\n:::";
        let output = protect_colon_block_plugins(input);
        assert!(output.contains("COLON_BLOCK_PLUGIN:alert:warning:"));
        assert!(!output.contains(":::"));
    }

    #[test]
    fn test_protect_colon_block_plugin_no_args() {
        let input = ":::toc\nignored\n:::";
        let output = protect_colon_block_plugins(input);
        assert!(output.contains("COLON_BLOCK_PLUGIN:toc::"));
    }

    #[test]
    fn test_protect_colon_block_plugin_empty_content() {
        let input = ":::clear\n:::";
        let output = protect_colon_block_plugins(input);
        assert!(output.contains("COLON_BLOCK_PLUGIN:clear::"));
        assert!(output.contains("COLON_BLOCK_PLUGIN}}"));
    }

    #[test]
    fn test_protect_colon_block_plugin_multiline_content() {
        let input = ":::box\nline one\nline two\n:::";
        let output = protect_colon_block_plugins(input);

        // Decode the base64 content back out and verify roundtrip
        let marker_re = Regex::new(r"COLON_BLOCK_PLUGIN:box::([^:]+):COLON_BLOCK_PLUGIN").unwrap();
        let caps = marker_re
            .captures(&output)
            .expect("marker should be present");
        let decoded = general_purpose::STANDARD.decode(&caps[1]).unwrap();
        assert_eq!(String::from_utf8(decoded).unwrap(), "line one\nline two");
    }

    #[test]
    fn test_protect_colon_block_plugin_unclosed_left_untouched() {
        let input = ":::alert warning\nSomething went wrong";
        let output = protect_colon_block_plugins(input);
        // No closing marker present, so the input is left unchanged
        assert_eq!(output, input);
    }

    #[test]
    fn test_protect_colon_block_plugin_nested_not_supported() {
        // The outer block closes at the FIRST bare `:::` line, which
        // belongs to the inner (nested) block. The nested start marker
        // is therefore captured as opaque literal content, and the
        // outer's real closing `:::` is left over as plain text.
        let input = ":::outer args\n:::inner args2\ncontent\n:::\n:::";
        let output = protect_colon_block_plugins(input);
        assert!(output.contains("COLON_BLOCK_PLUGIN:outer:args:"));
        // The leftover closing line for the (never-matched) outer block remains
        assert!(output.trim_end().ends_with(":::"));
    }

    #[test]
    fn test_restore_clear_from_colon_block() {
        let protected = protect_colon_block_plugins(":::clear\n:::");
        let output = restore_markers(&protected);
        assert_eq!(output, "<div class=\"clearfix\"></div>");
    }

    #[test]
    fn test_restore_generic_template_fallback() {
        let protected = protect_block_plugins("@youtube(dQw4w9WgXcQ)");
        let output = restore_markers(&protected);
        assert!(output.contains("<template class=\"umd-plugin umd-plugin-youtube\">"));
        assert!(output.contains("<data value=\"0\">dQw4w9WgXcQ</data>"));
    }
}
