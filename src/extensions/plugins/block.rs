//! Block plugin notation: `@function(args){{content}}` / `@function(args){content}`
//! / `@function(args)`.
//!
//! Like `plugins::inline`, this module owns both the "protect" step
//! (`protect_block_plugins`, run before Markdown parsing) and the "restore"
//! step (`restore_markers`, run during
//! `conflict_resolver::postprocess_conflicts_with_options`) for `@`-notation
//! block-level plugin syntax. Recognized ("standard") function names
//! (`math`, `popover`, `clear`) get real HTML; anything else falls back to a
//! generic `<template class="umd-plugin umd-plugin-block umd-plugin-*">`.
//!
//! The `::: 記法` colon-fence variant is, strictly speaking, fence notation
//! rather than plugin notation: its parsing (finding the opening/closing
//! lines, capturing raw content) lives in `fence::plugin_block`, which calls
//! into [`render_fence_block_plugin`] here for the actual per-function
//! rendering.

use base64::{Engine as _, engine::general_purpose};
use regex::{Captures, Regex};

use super::{escape_html_text, render_args_as_data, render_math_html, render_popover_html};

/// Attempt to parse one `@`-notation block plugin call starting at
/// `chars[start]` (which must be `@`).
///
/// On success, returns the index to resume scanning from (just past the
/// call) and the marker text that replaces it; `None` means `start` isn't
/// the beginning of a recognized call (an unrelated `@`, or a function
/// name not immediately followed by `(args)` — block plugins have no
/// bare/parens-less form), and the caller copies the `@` through unchanged.
///
/// Like [`super::inline::parse_inline_plugin_at`], this walks delimiters
/// by hand with [`super::scan_balanced`] / [`super::scan_balanced_double`]
/// instead of the three whole-text regex passes this used to be. The old
/// multiline regex used a *lazy* match (`[\s\S]*?`) that stopped at the
/// first `}}`, so two sibling block plugins in the same `{{...}}` — or one
/// nested inside another's content — would have the first one's content
/// swallow only up to its neighbor's closing `}}`, corrupting both. Proper
/// depth tracking on `{{`/`}}` fixes that. As with the inline side, a
/// plugin's `args`/`content` is still captured as opaque text, not
/// reparsed for nested calls or Markdown — see the "known issues" section
/// in `docs/runtime-features.md`.
fn parse_block_plugin_at(chars: &[char], start: usize) -> Option<(usize, String)> {
    debug_assert_eq!(chars[start], '@');
    let mut i = start + 1;
    let name_start = i;
    while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
        i += 1;
    }
    if i == name_start {
        return None;
    }
    let function: String = chars[name_start..i].iter().collect();

    // Block plugins have no bare form — `(args)` is always required.
    if chars.get(i) != Some(&'(') {
        return None;
    }
    let paren_close = super::scan_balanced(chars, i, '(', ')')?;
    let args: String = chars[i + 1..paren_close].iter().collect();
    let after_args = paren_close + 1;

    // Multiline content: @function(args){{ content }}
    if chars.get(after_args) == Some(&'{') && chars.get(after_args + 1) == Some(&'{') {
        if let Some(content_close) = super::scan_balanced_double(chars, after_args) {
            let content: String = chars[after_args + 2..content_close].iter().collect();
            let encoded_content = general_purpose::STANDARD.encode(content.as_bytes());
            let marker = format!(
                "{{{{BLOCK_PLUGIN:{}:{}:{}:BLOCK_PLUGIN}}}}",
                function, args, encoded_content
            );
            return Some((content_close + 2, marker));
        }
    }

    // Single-line content: @function(args){content}
    if chars.get(after_args) == Some(&'{') {
        if let Some(content_close) = super::scan_balanced(chars, after_args, '{', '}') {
            let content: String = chars[after_args + 1..content_close].iter().collect();
            let encoded_content = general_purpose::STANDARD.encode(content.as_bytes());
            let marker = format!(
                "{{{{BLOCK_PLUGIN:{}:{}:{}:BLOCK_PLUGIN}}}}",
                function, args, encoded_content
            );
            return Some((content_close + 1, marker));
        }
    }

    // Args only: @function(args)
    let encoded_args = general_purpose::STANDARD.encode(args.as_bytes());
    let marker = format!(
        "{{{{BLOCK_PLUGIN_ARGSONLY:{}:{}:BLOCK_PLUGIN_ARGSONLY}}}}",
        function, encoded_args
    );
    Some((after_args, marker))
}

/// Protect block plugin syntax by converting to markers
///
/// Converts various block plugin patterns into safe markers:
/// - `@function(args){{ content }}` → marker with content
/// - `@function(args){content}` → marker with content
/// - `@function(args)` → marker with args
///
/// Implemented as a single left-to-right scan (see
/// [`parse_block_plugin_at`]) rather than three sequential whole-text regex
/// passes — see that function's doc comment for why.
pub fn protect_block_plugins(input: &str) -> String {
    let chars: Vec<char> = input.chars().collect();
    let mut result = String::with_capacity(input.len());
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '@' {
            if let Some((next_i, marker)) = parse_block_plugin_at(&chars, i) {
                result.push_str(&marker);
                i = next_i;
                continue;
            }
        }
        result.push(chars[i]);
        i += 1;
    }
    result
}

/// Restore `{{BLOCK_PLUGIN...}}` markers left by `protect_block_plugins`.
///
/// Called from `conflict_resolver::postprocess_conflicts_with_options`
/// alongside `fence::plugin_block::restore_markers` — see that function for
/// the surrounding restore order.
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
                    "<template class=\"umd-plugin umd-plugin-block umd-plugin-{}\">{}</template>",
                    function, args_html
                )
            } else {
                format!(
                    "<template class=\"umd-plugin umd-plugin-block umd-plugin-{}\">{}{}</template>",
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
                "<template class=\"umd-plugin umd-plugin-block umd-plugin-{}\">{}</template>",
                function, args_html
            )
        })
        .to_string();

    result
}

/// Render the HTML for a `::: 記法` fence block-plugin invocation, given the
/// already-decoded function name, raw args, and content.
///
/// Called from `fence::plugin_block::restore_markers`, which owns the fence
/// parsing (finding the opening/closing lines and base64-decoding the
/// captured content) — this function owns only the per-function rendering
/// decision, shared conceptually with `restore_markers` above.
pub(crate) fn render_fence_block_plugin(function: &str, args: &str, content: &str) -> String {
    if function == "clear" && args.trim().is_empty() && content.trim().is_empty() {
        return "<div class=\"clearfix\"></div>".to_string();
    }

    if function == "math" {
        let formula = if content.trim().is_empty() {
            args
        } else {
            content
        };
        if let Some(mathml) = render_math_html(formula, true) {
            return mathml;
        }
    }

    if function == "popover" {
        return render_popover_html(args, content);
    }

    let args_html = render_args_as_data(args);
    let escaped_content = escape_html_text(content);

    if escaped_content.is_empty() {
        format!(
            "<template class=\"umd-plugin umd-plugin-block umd-plugin-{}\">{}</template>",
            function, args_html
        )
    } else {
        format!(
            "<template class=\"umd-plugin umd-plugin-block umd-plugin-{}\">{}{}</template>",
            function, args_html, escaped_content
        )
    }
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

    // Regression tests for the balanced-bracket rewrite of
    // `protect_block_plugins` (2026-09) -- the previous multiline regex
    // used a *lazy* match that stopped at the first `}}`, so nested or
    // sibling block plugins inside a `{{...}}` corrupted each other. See
    // `inline.rs`'s equivalent regression tests, and the "known issues"
    // section in `docs/runtime-features.md`, for the same nuance here:
    // args/content are captured as opaque text, not reparsed for nested
    // calls or Markdown -- only the marker-corruption bug is fixed here.

    #[test]
    fn test_protect_block_plugin_nested_multiline() {
        // A block plugin nested inside another's multiline content. The
        // old lazy `[\s\S]*?` regex would have matched the *inner*
        // plugin's own `}}` as the outer's closing delimiter, truncating
        // the outer content and leaving the true closing `}}` as leftover
        // literal text.
        let input = "@tab(One){{ @tab-content(A){{ inner content }} more }}";
        let output = protect_block_plugins(input);

        assert!(output.starts_with("{{BLOCK_PLUGIN:tab:One:"));
        assert!(output.ends_with(":BLOCK_PLUGIN}}"));
        assert_eq!(output.matches("BLOCK_PLUGIN:").count(), 1);

        let encoded = output
            .trim_start_matches("{{BLOCK_PLUGIN:tab:One:")
            .trim_end_matches(":BLOCK_PLUGIN}}");
        let decoded = String::from_utf8(
            general_purpose::STANDARD.decode(encoded).unwrap(),
        )
        .unwrap();
        assert_eq!(decoded, " @tab-content(A){{ inner content }} more ");
    }

    #[test]
    fn test_protect_block_plugin_sibling_multiline_blocks() {
        // Two sibling (not nested) multiline block plugins back to back.
        // The old lazy regex still handled this particular shape, but it's
        // exercised here to lock in the new scanner's behavior alongside
        // the nested case above.
        let input = "@tab(One){{ first }}
@tab(Two){{ second }}";
        let output = protect_block_plugins(input);
        assert_eq!(output.matches("BLOCK_PLUGIN:").count(), 2);
        assert!(output.contains("{{BLOCK_PLUGIN:tab:One:"));
        assert!(output.contains("{{BLOCK_PLUGIN:tab:Two:"));
    }

    #[test]
    fn test_protect_block_plugin_args_with_nested_parens() {
        // Args containing balanced parens (e.g. a Markdown link) previously
        // broke the args capture, which used `[^)]*` and so stopped at the
        // first `)` regardless of nesting.
        let input = "@cite([Source](https://example.com/)){{ content }}";
        let output = protect_block_plugins(input);
        assert!(
            output.starts_with("{{BLOCK_PLUGIN:cite:[Source](https://example.com/):"),
            "expected full args with nested link parens preserved, got: {}",
            output
        );
    }

    #[test]
    fn test_protect_block_plugin_unclosed_content_falls_back_to_args_only() {
        // Content that never closes doesn't panic or hang. Unlike the
        // inline side, the block args-only form has no trailing-delimiter
        // requirement of its own (see `test_restore_generic_template_fallback`,
        // which uses bare `@youtube(id)` with no trailing punctuation at
        // all), so once `(args)` is found, an unterminated `{{`/`{` after
        // it falls back to an args-only match -- exactly like the original
        // regex cascade, whose multiline/single-line passes would also
        // fail to match here (no closing `}}`/`}` exists), leaving its
        // args-only pass to match `@tab(One)` alone.
        let input = "@tab(One){{ unterminated";
        let output = protect_block_plugins(input);
        assert_eq!(
            output,
            "{{BLOCK_PLUGIN_ARGSONLY:tab:T25l:BLOCK_PLUGIN_ARGSONLY}}{{ unterminated"
        );
    }

    #[test]
    fn test_protect_block_plugin_unclosed_args_is_left_literal() {
        // Unlike unterminated *content*, unterminated *args* (no closing
        // `)` at all) leaves nothing for any form to match against, so the
        // whole call is left as literal text.
        let input = "@tab(unterminated";
        let output = protect_block_plugins(input);
        assert_eq!(output, input);
    }

    #[test]
    fn test_protect_block_plugin_no_bare_form() {
        // Unlike inline plugins, block plugins have no bare `@function;`
        // form -- `(args)` is always required, so a plain `@word` is left
        // untouched.
        let input = "Email me @someone about this.";
        let output = protect_block_plugins(input);
        assert_eq!(output, input);
    }

    #[test]
    fn test_restore_generic_template_fallback() {
        let protected = protect_block_plugins("@youtube(dQw4w9WgXcQ)");
        let output = restore_markers(&protected);
        assert!(output.contains("<template class=\"umd-plugin umd-plugin-block umd-plugin-youtube\">"));
        assert!(output.contains("<data value=\"0\">dQw4w9WgXcQ</data>"));
    }
}
