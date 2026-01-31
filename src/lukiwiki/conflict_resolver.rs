//! Syntax conflict resolution for LukiWiki and Markdown
//!
//! This module handles cases where LukiWiki and Markdown syntax might conflict.
//! The general strategy is:
//! 1. Process input before Markdown parsing (pre-processing)
//! 2. Apply LukiWiki-specific transformations after Markdown rendering (post-processing)
//! 3. Use distinctive markers to avoid ambiguous patterns

use once_cell::sync::Lazy;
use regex::{Captures, Regex};
use std::collections::HashMap;

/// Convert comma-separated args to JSON array
///
/// # Arguments
///
/// * `args` - Comma-separated argument string
///
/// # Returns
///
/// JSON array string
fn args_to_json(args: &str) -> String {
    if args.is_empty() {
        return "[]".to_string();
    }

    let parts: Vec<String> = args
        .split(',')
        .map(|s| {
            let trimmed = s.trim();
            // Escape quotes and backslashes in the string
            let escaped = trimmed.replace('\\', "\\\\").replace('"', "\\\"");
            format!("\"{}\"", escaped)
        })
        .collect();

    format!("[{}]", parts.join(","))
}

// Patterns that need special handling

/// Regex to detect LukiWiki blockquote: > ... <
static LUKIWIKI_BLOCKQUOTE: Lazy<Regex> = Lazy::new(|| {
    // Match single line > content < pattern
    Regex::new(r"(?m)^>\s*(.+?)\s*<\s*$").unwrap()
});

/// Regex to detect Markdown-style emphasis that might conflict with LukiWiki
/// Detects ***text*** which could be confused with '''text'''
static TRIPLE_STAR_EMPHASIS: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\*\*\*([^*]+)\*\*\*").unwrap());

/// Regex to detect custom header ID syntax: # Header {#custom-id}
static CUSTOM_HEADER_ID: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?m)^(#{1,6})\s+(.+?)\s+\{#([a-zA-Z0-9_-]+)\}\s*$").unwrap());

/// Store custom header IDs during preprocessing
#[derive(Debug, Clone)]
pub struct HeaderIdMap {
    /// Maps heading number (1-based) to custom ID
    pub ids: HashMap<usize, String>,
}

impl HeaderIdMap {
    pub fn new() -> Self {
        Self {
            ids: HashMap::new(),
        }
    }
}

/// Pre-process input to resolve conflicts before Markdown parsing
///
/// This function escapes or transforms syntax that would otherwise create
/// ambiguous parsing situations. It also extracts custom header IDs.
///
/// # Arguments
///
/// * `input` - The raw wiki markup input
///
/// # Returns
///
/// A tuple of (pre-processed markup, header ID map)
///
/// # Examples
///
/// ```
/// use lukiwiki_parser::lukiwiki::conflict_resolver::preprocess_conflicts;
///
/// let input = "> quote <";
/// let (output, _) = preprocess_conflicts(input);
/// // LukiWiki blockquote is preserved
/// ```
pub fn preprocess_conflicts(input: &str) -> (String, HeaderIdMap) {
    let mut result = input.to_string();
    let mut header_map = HeaderIdMap::new();
    let mut heading_counter = 0;

    // Extract custom header IDs: # Header {#custom-id}
    result = CUSTOM_HEADER_ID
        .replace_all(&result, |caps: &Captures| {
            heading_counter += 1;
            let hashes = &caps[1];
            let title = &caps[2];
            let custom_id = &caps[3];

            // Store the custom ID for this heading
            header_map
                .ids
                .insert(heading_counter, custom_id.to_string());

            // Return the heading without the {#id} part
            format!("{} {}", hashes, title)
        })
        .to_string();

    // Handle LukiWiki blockquotes: > ... <
    // Use a safe marker that won't be affected by HTML escaping
    result = LUKIWIKI_BLOCKQUOTE
        .replace_all(&result, |caps: &Captures| {
            let content = &caps[1];
            format!(
                "{{{{LUKIWIKI_BLOCKQUOTE:{}:LUKIWIKI_BLOCKQUOTE}}}}",
                content
            )
        })
        .to_string();

    // Protect LukiWiki block decorations (COLOR, SIZE, alignment)
    // These will be applied in post-processing
    let color_prefix = Regex::new(r"(?m)^(COLOR\([^)]*\):\s*.+)$").unwrap();
    result = color_prefix
        .replace_all(&result, |caps: &Captures| {
            format!("{{{{BLOCK_DECORATION:{}:BLOCK_DECORATION}}}}", &caps[1])
        })
        .to_string();

    let size_prefix = Regex::new(r"(?m)^(SIZE\([^)]+\):\s*.+)$").unwrap();
    result = size_prefix
        .replace_all(&result, |caps: &Captures| {
            format!("{{{{BLOCK_DECORATION:{}:BLOCK_DECORATION}}}}", &caps[1])
        })
        .to_string();

    let align_prefix = Regex::new(r"(?m)^((RIGHT|CENTER|LEFT):\s*.+)$").unwrap();
    result = align_prefix
        .replace_all(&result, |caps: &regex::Captures| {
            format!("{{{{BLOCK_DECORATION:{}:BLOCK_DECORATION}}}}", &caps[1])
        })
        .to_string();

    // Protect inline plugins with content but no args: &function{content};
    // Use base64 encoding to safely preserve content with special characters
    let inline_plugin_noargs_content = Regex::new(r"&(\w+)\{((?:[^{}]|\{[^}]*\})*)\};").unwrap();
    result = inline_plugin_noargs_content
        .replace_all(&result, |caps: &regex::Captures| {
            use base64::{Engine as _, engine::general_purpose};
            let function = &caps[1];
            let content = &caps[2];
            let encoded_content = general_purpose::STANDARD.encode(content.as_bytes());
            format!(
                "{{{{INLINE_PLUGIN:{}::{}:INLINE_PLUGIN}}}}",
                function, encoded_content
            )
        })
        .to_string();

    // Protect inline plugins: &function(args){content};
    // Use base64 encoding to safely preserve content with special characters
    // Note: Inline decoration functions ARE protected here and processed in postprocess
    let inline_plugin = Regex::new(r"&(\w+)\(([^)]*)\)\{((?:[^{}]|\{[^}]*\})*)\};").unwrap();
    result = inline_plugin
        .replace_all(&result, |caps: &regex::Captures| {
            use base64::{Engine as _, engine::general_purpose};
            let function = &caps[1];
            let args = &caps[2];
            let content = &caps[3];
            let encoded_content = general_purpose::STANDARD.encode(content.as_bytes());
            format!(
                "{{{{INLINE_PLUGIN:{}:{}:{}:INLINE_PLUGIN}}}}",
                function, args, encoded_content
            )
        })
        .to_string();

    // Protect inline plugins (args only): &function(args);
    let inline_plugin_argsonly = Regex::new(r"&(\w+)\(([^)]*)\);").unwrap();
    result = inline_plugin_argsonly
        .replace_all(&result, |caps: &regex::Captures| {
            let function = &caps[1];
            let args = &caps[2];
            format!(
                "{{{{INLINE_PLUGIN_ARGSONLY:{}:{}:INLINE_PLUGIN_ARGSONLY}}}}",
                function, args
            )
        })
        .to_string();

    // Protect inline plugins (no args): &function;
    let inline_plugin_noargs = Regex::new(r"&(\w+);").unwrap();
    result = inline_plugin_noargs
        .replace_all(&result, |caps: &regex::Captures| {
            let function = &caps[1];
            format!(
                "{{{{INLINE_PLUGIN_NOARGS:{}:INLINE_PLUGIN_NOARGS}}}}",
                function
            )
        })
        .to_string();

    // Protect block plugins multiline: @function(args){{ content }}
    // Use base64 encoding and markers to preserve content
    let block_plugin_multi = Regex::new(r"@(\w+)\(([^)]*)\)\{\{([\s\S]*?)\}\}").unwrap();
    result = block_plugin_multi
        .replace_all(&result, |caps: &regex::Captures| {
            use base64::{Engine as _, engine::general_purpose};
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
        .replace_all(&result, |caps: &Captures| {
            use base64::{Engine as _, engine::general_purpose};
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
    // This should be processed AFTER patterns with { and {{
    let block_plugin_argsonly = Regex::new(r"@(\w+)\(([^)]*)\)").unwrap();
    result = block_plugin_argsonly
        .replace_all(&result, |caps: &regex::Captures| {
            use base64::{Engine as _, engine::general_purpose};
            let function = &caps[1];
            let args = &caps[2];
            // Encode args to prevent Markdown parser from converting URLs
            let encoded_args = general_purpose::STANDARD.encode(args.as_bytes());
            format!(
                "{{{{BLOCK_PLUGIN_ARGSONLY:{}:{}:BLOCK_PLUGIN_ARGSONLY}}}}",
                function, encoded_args
            )
        })
        .to_string();

    (result, header_map)
}

/// Post-process HTML to restore LukiWiki-specific syntax and apply custom header IDs
///
/// This function converts temporary markers back to their intended HTML output
/// and replaces sequential header IDs with custom IDs where specified.
///
/// # Arguments
///
/// * `html` - The HTML output from Markdown parser
/// * `header_map` - Map of custom header IDs
///
/// # Returns
///
/// HTML with LukiWiki blockquotes properly rendered and custom IDs applied
/// Convert inline decoration function to HTML
/// Returns None if not a decoration function
fn convert_inline_decoration_to_html(function: &str, args: &str, content: &str) -> Option<String> {
    match function {
        // Simple wrapper tags without content
        "dfn" => Some(format!("<dfn>{}</dfn>", content)),
        "kbd" => Some(format!("<kbd>{}</kbd>", content)),
        "samp" => Some(format!("<samp>{}</samp>", content)),
        "var" => Some(format!("<var>{}</var>", content)),
        "cite" => Some(format!("<cite>{}</cite>", content)),
        "q" => Some(format!("<q>{}</q>", content)),
        "small" => Some(format!("<small>{}</small>", content)),
        "u" => Some(format!("<u>{}</u>", content)),
        "bdi" => Some(format!("<bdi>{}</bdi>", content)),

        // Tags with attributes
        "ruby" => {
            // &ruby(reading){text}; → <ruby>text<rp>(</rp><rt>reading</rt><rp>)</rp></ruby>
            Some(format!(
                "<ruby>{}<rp>(</rp><rt>{}</rt><rp>)</rp></ruby>",
                content, args
            ))
        }
        "time" => {
            // &time(datetime){text}; → <time datetime="datetime">text</time>
            Some(format!("<time datetime=\"{}\">{}</time>", args, content))
        }
        "data" => {
            // &data(value){text}; → <data value="value">text</data>
            Some(format!("<data value=\"{}\">{}</data>", args, content))
        }
        "bdo" => {
            // &bdo(dir){text}; → <bdo dir="dir">text</bdo>
            Some(format!("<bdo dir=\"{}\">{}</bdo>", args, content))
        }
        "lang" => {
            // &lang(locale){text}; → <span lang="locale">text</span>
            Some(format!("<span lang=\"{}\">{}</span>", args, content))
        }
        "abbr" => {
            // &abbr(text){description}; → <abbr title="description">text</abbr>
            Some(format!("<abbr title=\"{}\">{}</abbr>", content, args))
        }
        "sup" => {
            // &sup(text); → <sup>text</sup>
            Some(format!("<sup>{}</sup>", args))
        }
        "sub" => {
            // &sub(text); → <sub>text</sub>
            Some(format!("<sub>{}</sub>", args))
        }
        "color" => {
            // &color(fg,bg){text}; → <span style="color: fg; background-color: bg">text</span>
            let parts: Vec<&str> = args.split(',').collect();
            let fg = parts.get(0).map(|s| s.trim()).unwrap_or("");
            let bg = parts.get(1).map(|s| s.trim()).unwrap_or("");

            let mut styles = Vec::new();
            if !fg.is_empty() && fg != "inherit" {
                styles.push(format!("color: {}", fg));
            }
            if !bg.is_empty() && bg != "inherit" {
                styles.push(format!("background-color: {}", bg));
            }

            if styles.is_empty() {
                Some(content.to_string())
            } else {
                Some(format!(
                    "<span style=\"{}\">{}</span>",
                    styles.join("; "),
                    content
                ))
            }
        }
        "size" => {
            // &size(rem){text}; → <span style="font-size: remrem">text</span>
            Some(format!(
                "<span style=\"font-size: {}rem\">{}</span>",
                args, content
            ))
        }
        _ => None,
    }
}

/// Convert args-only inline decoration function to HTML
fn convert_inline_decoration_argsonly_to_html(function: &str, args: &str) -> Option<String> {
    match function {
        "sup" => Some(format!("<sup>{}</sup>", args)),
        "sub" => Some(format!("<sub>{}</sub>", args)),
        _ => None,
    }
}

/// Convert no-args inline decoration function to HTML
fn convert_inline_decoration_noargs_to_html(function: &str) -> Option<String> {
    match function {
        "wbr" => Some("<wbr />".to_string()),
        "br" => Some("<br />".to_string()),
        _ => None,
    }
}

pub fn postprocess_conflicts(html: &str, header_map: &HeaderIdMap) -> String {
    use crate::lukiwiki::block_decorations;

    let mut result = html.to_string();

    // Add header IDs: <h1>Title</h1> -> <h1><a href="#id" id="id"></a>Title</h1>
    let mut heading_counter = 0;
    let header_regex = Regex::new(r"<h([1-6])>([^<]+)</h([1-6])>").unwrap();
    result = header_regex
        .replace_all(&result, |caps: &Captures| {
            heading_counter += 1;
            let level = &caps[1];
            let title = &caps[2];
            let close_level = &caps[3];

            let id = if let Some(custom_id) = header_map.ids.get(&heading_counter) {
                // Add 'h-' prefix to custom IDs to avoid conflicts with system IDs
                format!("h-{}", custom_id)
            } else {
                // Auto-numbered IDs also use 'h-' prefix for consistency
                format!("h-{}", heading_counter)
            };

            format!(
                "<h{}><a href=\"#{}\" aria-hidden=\"true\" class=\"anchor\" id=\"{}\"></a>{}</h{}>",
                level, id, id, title, close_level
            )
        })
        .to_string();

    // Restore LukiWiki blockquotes
    let lukiwiki_blockquote_marker =
        Regex::new(r"\{\{LUKIWIKI_BLOCKQUOTE:(.+?):LUKIWIKI_BLOCKQUOTE\}\}").unwrap();

    result = lukiwiki_blockquote_marker
        .replace_all(&result, |caps: &Captures| {
            let content = &caps[1];
            format!("<blockquote class=\"lukiwiki\">{}</blockquote>", content)
        })
        .to_string();

    // Restore and apply block decorations
    let block_decoration_marker =
        Regex::new(r"<p>\{\{BLOCK_DECORATION:(.+?):BLOCK_DECORATION\}\}</p>").unwrap();

    result = block_decoration_marker
        .replace_all(&result, |caps: &Captures| {
            let decoration = &caps[1];
            // Apply block decoration logic
            block_decorations::apply_block_decorations(decoration)
        })
        .to_string();

    // Restore inline plugins
    let inline_plugin_marker =
        Regex::new(r"\{\{INLINE_PLUGIN:(\w+):([^:]*):([^:]*):INLINE_PLUGIN\}\}").unwrap();
    result = inline_plugin_marker
        .replace_all(&result, |caps: &Captures| {
            use base64::{Engine as _, engine::general_purpose};
            let function = &caps[1];
            let args = &caps[2];
            let encoded_content = &caps[3];

            // Decode base64 to get original content
            let content = general_purpose::STANDARD
                .decode(encoded_content.as_bytes())
                .ok()
                .and_then(|bytes| String::from_utf8(bytes).ok())
                .unwrap_or_else(|| encoded_content.to_string());

            // Try to convert as inline decoration function
            if let Some(html) = convert_inline_decoration_to_html(function, args, &content) {
                return html;
            }

            // Otherwise, convert to plugin HTML
            // Escape HTML entities in content while preserving & for nested plugins
            let escaped_content = content.replace('<', "&lt;").replace('>', "&gt;");

            // Convert args to JSON array
            let json_args = args_to_json(args);

            format!(
                "<span class=\"plugin-{}\" data-args='{}'>{}</span>",
                function, json_args, escaped_content
            )
        })
        .to_string();

    // Restore inline plugins (args only)
    let inline_plugin_argsonly_marker =
        Regex::new(r"\{\{INLINE_PLUGIN_ARGSONLY:(\w+):([^:]*):INLINE_PLUGIN_ARGSONLY\}\}").unwrap();
    result = inline_plugin_argsonly_marker
        .replace_all(&result, |caps: &Captures| {
            let function = &caps[1];
            let args = &caps[2];

            // Try to convert as inline decoration function
            if let Some(html) = convert_inline_decoration_argsonly_to_html(function, args) {
                return html;
            }

            // Otherwise, convert to plugin HTML
            let json_args = args_to_json(args);

            format!(
                "<span class=\"plugin-{}\" data-args='{}' />",
                function, json_args
            )
        })
        .to_string();

    // Restore inline plugins (no args)
    let inline_plugin_noargs_marker =
        Regex::new(r"\{\{INLINE_PLUGIN_NOARGS:(\w+):INLINE_PLUGIN_NOARGS\}\}").unwrap();
    result = inline_plugin_noargs_marker
        .replace_all(&result, |caps: &Captures| {
            let function = &caps[1];

            // Try to convert as inline decoration function
            if let Some(html) = convert_inline_decoration_noargs_to_html(function) {
                return html;
            }

            // Otherwise, convert to plugin HTML
            format!("<span class=\"plugin-{}\" data-args='[]' />", function)
        })
        .to_string();

    // Restore block plugins
    let block_plugin_marker =
        Regex::new(r"\{\{BLOCK_PLUGIN:(\w+):([^:]*):([^:]*):BLOCK_PLUGIN\}\}").unwrap();
    result = block_plugin_marker
        .replace_all(&result, |caps: &Captures| {
            use base64::{Engine as _, engine::general_purpose};
            let function = &caps[1];
            let args = &caps[2];
            let encoded_content = &caps[3];
            // Decode base64 to get original content
            let content = general_purpose::STANDARD
                .decode(encoded_content.as_bytes())
                .ok()
                .and_then(|bytes| String::from_utf8(bytes).ok())
                .unwrap_or_else(|| encoded_content.to_string());

            // Escape HTML entities in content while preserving & for nested plugins
            let escaped_content = content.replace('<', "&lt;").replace('>', "&gt;");

            // Convert args to JSON array
            let json_args = args_to_json(args);

            format!(
                "<div class=\"plugin-{}\" data-args='{}'>{}</div>",
                function, json_args, escaped_content
            )
        })
        .to_string();

    // Restore block plugins (args only, no content)
    let block_plugin_argsonly_marker =
        Regex::new(r"\{\{BLOCK_PLUGIN_ARGSONLY:(\w+):([^:]*):BLOCK_PLUGIN_ARGSONLY\}\}").unwrap();
    result = block_plugin_argsonly_marker
        .replace_all(&result, |caps: &Captures| {
            use base64::{Engine as _, engine::general_purpose};
            let function = &caps[1];
            let encoded_args = &caps[2];
            // Decode base64 to get original args
            let args = general_purpose::STANDARD
                .decode(encoded_args.as_bytes())
                .ok()
                .and_then(|bytes| String::from_utf8(bytes).ok())
                .unwrap_or_else(|| encoded_args.to_string());
            let json_args = args_to_json(&args);

            format!(
                "<div class=\"plugin-{}\" data-args='{}' />",
                function, json_args
            )
        })
        .to_string();

    // Remove wrapping <p> tags around block plugins
    let wrapped_plugin =
        Regex::new(r#"<p>\s*(<div class="plugin-[^"]+"[^>]*>.*?</div>)\s*</p>"#).unwrap();
    result = wrapped_plugin.replace_all(&result, "$1").to_string();

    // Remove wrapping <p> tags around self-closing block plugins
    let wrapped_plugin_self =
        Regex::new(r#"<p>\s*(<div class="plugin-[^"]+"[^>]*/>\s*)\s*</p>"#).unwrap();
    result = wrapped_plugin_self.replace_all(&result, "$1").to_string();

    // Apply Bootstrap default classes and GFM alerts
    result = apply_bootstrap_enhancements(&result);

    result
}

/// Apply Bootstrap 5 enhancements to HTML
///
/// - Add default `table` class to all <table> elements
/// - Add default `blockquote` class to all <blockquote> elements (except LukiWiki-style)
/// - Convert GFM alerts ([!NOTE], etc.) to Bootstrap alert components
/// - Add JUSTIFY support for tables (w-100 class)
fn apply_bootstrap_enhancements(html: &str) -> String {
    let mut result = html.to_string();

    // Add default class to tables
    let table_pattern = Regex::new(r"<table>").unwrap();
    result = table_pattern
        .replace_all(&result, "<table class=\"table\">")
        .to_string();

    // Add default class to blockquotes (check if it doesn't already have class="lukiwiki")
    let blockquote_pattern = Regex::new(r#"<blockquote>"#).unwrap();
    result = blockquote_pattern
        .replace_all(&result, "<blockquote class=\"blockquote\">")
        .to_string();

    // LukiWiki blockquotes already have class="lukiwiki", so they remain unchanged

    // Handle GFM alerts: > [!NOTE] etc.
    // These are rendered as <blockquote class="blockquote"><p>[!NOTE] ...</p></blockquote>
    let gfm_alert_pattern = Regex::new(
        r#"<blockquote class="blockquote">\s*<p>\[!(NOTE|TIP|IMPORTANT|WARNING|CAUTION)\]\s*(.*?)</p>\s*</blockquote>"#
    ).unwrap();

    result = gfm_alert_pattern
        .replace_all(&result, |caps: &Captures| {
            let alert_type = &caps[1];
            let content = &caps[2];

            let (alert_class, icon_text) = match alert_type {
                "NOTE" => ("alert-info", "Note"),
                "TIP" => ("alert-success", "Tip"),
                "IMPORTANT" => ("alert-primary", "Important"),
                "WARNING" => ("alert-warning", "Warning"),
                "CAUTION" => ("alert-danger", "Caution"),
                _ => ("alert-info", "Note"),
            };

            format!(
                r#"<div class="alert {}" role="alert"><strong>{}:</strong> {}</div>"#,
                alert_class, icon_text, content
            )
        })
        .to_string();

    result
}

/// Check if input contains potentially ambiguous syntax
///
/// Used for diagnostics and warnings. Returns descriptions of
/// detected conflicts.
///
/// # Arguments
///
/// * `input` - The raw wiki markup input
///
/// # Returns
///
/// Vector of warning messages for ambiguous patterns
pub fn detect_ambiguous_syntax(input: &str) -> Vec<String> {
    let mut warnings = Vec::new();

    // Check for ***text*** which could be confused with '''text'''
    if TRIPLE_STAR_EMPHASIS.is_match(input) && input.contains("'''") {
        warnings.push(
            "Detected both ***text*** (Markdown) and '''text''' (LukiWiki). \
             Consider using **text** for Markdown bold-italic."
                .to_string(),
        );
    }

    // Check for potential COLOR(): vs Markdown definition list conflict
    if input.contains("COLOR(") && input.contains("\n:") {
        warnings.push(
            "Detected COLOR() syntax near Markdown definition list. \
             Ensure proper spacing to avoid ambiguity."
                .to_string(),
        );
    }

    warnings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lukiwiki_blockquote_preprocessing() {
        let input = "> This is a LukiWiki quote <";
        let (output, _) = preprocess_conflicts(input);
        assert!(output.contains("{{LUKIWIKI_BLOCKQUOTE:"));
        assert!(!output.starts_with(">"));
    }

    #[test]
    fn test_lukiwiki_blockquote_postprocessing() {
        let header_map = HeaderIdMap::new();
        let input = "{{LUKIWIKI_BLOCKQUOTE:Test content:LUKIWIKI_BLOCKQUOTE}}";
        let output = postprocess_conflicts(input, &header_map);
        assert!(output.contains("<blockquote class=\"lukiwiki\">Test content</blockquote>"));
    }

    #[test]
    fn test_markdown_blockquote_unchanged() {
        let input = "> Standard Markdown quote\n> Second line";
        let (output, _) = preprocess_conflicts(input);
        // Should NOT be converted (no closing <)
        assert_eq!(output, input);
    }

    #[test]
    fn test_roundtrip_blockquote() {
        let header_map = HeaderIdMap::new();
        let input = "> LukiWiki style <";
        let (preprocessed, _) = preprocess_conflicts(input);
        let postprocessed = postprocess_conflicts(&preprocessed, &header_map);
        assert!(postprocessed.contains("<blockquote class=\"lukiwiki\">"));
    }

    #[test]
    fn test_custom_header_id() {
        let input = "# My Header {#custom-id}\n\nContent";
        let (output, header_map) = preprocess_conflicts(input);
        // Should extract the custom ID
        assert_eq!(header_map.ids.get(&1), Some(&"custom-id".to_string()));
        // Should remove {#custom-id} from the text
        assert!(!output.contains("{#custom-id}"));
        assert!(output.contains("# My Header"));
    }

    #[test]
    fn test_multiple_custom_header_ids() {
        let input = "# First {#first}\n\n## Second {#second}\n\n### Third";
        let (_output, header_map) = preprocess_conflicts(input);
        assert_eq!(header_map.ids.get(&1), Some(&"first".to_string()));
        assert_eq!(header_map.ids.get(&2), Some(&"second".to_string()));
        assert_eq!(header_map.ids.get(&3), None); // No custom ID for third
    }

    #[test]
    fn test_apply_custom_header_ids() {
        let mut header_map = HeaderIdMap::new();
        header_map.ids.insert(1, "my-custom-id".to_string());

        let html = "<h1>Header</h1>";
        let output = postprocess_conflicts(html, &header_map);

        assert!(output.contains("id=\"h-my-custom-id\""));
        assert!(output.contains("href=\"#h-my-custom-id\""));
        assert!(!output.contains("heading-1"));
    }

    #[test]
    fn test_sequential_header_ids() {
        let header_map = HeaderIdMap::new();
        let html = "<h1>First</h1><h2>Second</h2>";
        let output = postprocess_conflicts(html, &header_map);

        assert!(output.contains("id=\"h-1\""));
        assert!(output.contains("id=\"h-2\""));
    }

    #[test]
    fn test_detect_triple_emphasis_conflict() {
        let input = "***Markdown*** and '''LukiWiki'''";
        let warnings = detect_ambiguous_syntax(input);
        assert!(!warnings.is_empty());
        assert!(warnings[0].contains("***text***"));
    }

    #[test]
    fn test_detect_color_definition_conflict() {
        let input = "COLOR(red): text\n: definition";
        let warnings = detect_ambiguous_syntax(input);
        assert!(!warnings.is_empty());
        assert!(warnings[0].contains("COLOR()"));
    }

    #[test]
    fn test_no_warnings_for_clean_syntax() {
        let input = "# Heading\n\n**Bold** and ''LukiWiki bold''";
        let warnings = detect_ambiguous_syntax(input);
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_bootstrap_table_class() {
        let header_map = HeaderIdMap::new();
        let input = "<table><tr><td>Cell</td></tr></table>";
        let output = postprocess_conflicts(input, &header_map);
        assert!(output.contains(r#"<table class="table">"#));
    }

    #[test]
    fn test_bootstrap_blockquote_class() {
        let header_map = HeaderIdMap::new();
        let input = "<blockquote><p>Quote</p></blockquote>";
        let output = postprocess_conflicts(input, &header_map);
        assert!(output.contains(r#"<blockquote class="blockquote">"#));
    }

    #[test]
    fn test_gfm_alert_note() {
        let header_map = HeaderIdMap::new();
        let input = r#"<blockquote class="blockquote"><p>[!NOTE] This is a note</p></blockquote>"#;
        let output = postprocess_conflicts(input, &header_map);
        assert!(output.contains(r#"<div class="alert alert-info" role="alert">"#));
        assert!(output.contains("<strong>Note:</strong>"));
        assert!(output.contains("This is a note"));
    }

    #[test]
    fn test_gfm_alert_warning() {
        let header_map = HeaderIdMap::new();
        let input = r#"<blockquote class="blockquote"><p>[!WARNING] Be careful</p></blockquote>"#;
        let output = postprocess_conflicts(input, &header_map);
        assert!(output.contains(r#"<div class="alert alert-warning" role="alert">"#));
        assert!(output.contains("<strong>Warning:</strong>"));
    }

    #[test]
    fn test_lukiwiki_blockquote_no_bootstrap_class() {
        let header_map = HeaderIdMap::new();
        let input = "{{LUKIWIKI_BLOCKQUOTE:Test content:LUKIWIKI_BLOCKQUOTE}}";
        let output = postprocess_conflicts(input, &header_map);
        assert!(output.contains(r#"<blockquote class="lukiwiki">"#));
        assert!(!output.contains(r#"class="blockquote""#));
    }
}
