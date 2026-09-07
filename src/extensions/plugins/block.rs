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

    #[test]
    fn test_restore_generic_template_fallback() {
        let protected = protect_block_plugins("@youtube(dQw4w9WgXcQ)");
        let output = restore_markers(&protected);
        assert!(output.contains("<template class=\"umd-plugin umd-plugin-block umd-plugin-youtube\">"));
        assert!(output.contains("<data value=\"0\">dQw4w9WgXcQ</data>"));
    }
}
