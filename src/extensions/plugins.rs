//! Plugin system for LukiWiki
//!
//! Provides plugin syntax support:
//! - Inline plugins: &function(args){content};
//! - Block plugins (multiline): @function(args){{ content }}
//! - Block plugins (single line): @function(args){content}
//!
//! Note: This only parses plugin syntax and outputs placeholder HTML.
//! Actual plugin execution is handled by JavaScript/frontend layer.
//! Content within plugins may contain nested plugins or other Wiki syntax.

use once_cell::sync::Lazy;
use regex::Regex;
use base64::{engine::general_purpose, Engine as _};

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

/// Encode JSON args to base64 for safe HTML attribute storage
///
/// # Arguments
///
/// * `json_args` - JSON string to encode
///
/// # Returns
///
/// Base64 encoded string
fn encode_args(json_args: &str) -> String {
    general_purpose::STANDARD.encode(json_args.as_bytes())
}

// Block plugin patterns
static BLOCK_PLUGIN_MULTILINE: Lazy<Regex> = Lazy::new(|| {
    // Match @function(args){{ content }} using non-greedy match
    Regex::new(r"@(\w+)\(([^)]*)\)\{\{([\s\S]*?)\}\}").unwrap()
});

static BLOCK_PLUGIN_SINGLELINE: Lazy<Regex> = Lazy::new(|| {
    // Match @function(args){content} (single braces)
    Regex::new(r"@(\w+)\(([^)]*)\)\{([^}]*)\}").unwrap()
});

// Block plugin with args only (no content): @function(args)
static BLOCK_PLUGIN_ARGSONLY: Lazy<Regex> = Lazy::new(|| {
    // Match @function(args) - args only, no content
    // This should be processed AFTER patterns with { and {{
    Regex::new(r"@(\w+)\(([^)]*)\)").unwrap()
});

// Block plugin without args: @function()
static BLOCK_PLUGIN_NOARGS: Lazy<Regex> = Lazy::new(|| {
    // Match @function() - parens required to distinguish from @mentions
    Regex::new(r"@(\w+)\(\)").unwrap()
});

// Inline plugin pattern
static INLINE_PLUGIN: Lazy<Regex> = Lazy::new(|| {
    // Match &function(args){content};
    // Content may contain nested braces for nested plugins
    Regex::new(r"&(\w+)\(([^)]*)\)\{((?:[^{}]|\{[^}]*\})*)\};").unwrap()
});

// Inline plugin with args only: &function(args);
static INLINE_PLUGIN_ARGSONLY: Lazy<Regex> = Lazy::new(|| {
    // Match &function(args); (no content)
    Regex::new(r"&(\w+)\(([^)]*)\);").unwrap()
});

// Inline plugin without args: &function;
static INLINE_PLUGIN_NOARGS: Lazy<Regex> = Lazy::new(|| {
    // Match &function; (no args, no content)
    Regex::new(r"&(\w+);").unwrap()
});

/// Apply plugin syntax transformation
///
/// Converts plugin syntax to HTML containers that can be processed by JavaScript.
/// The parser only detects and preserves plugin metadata; actual execution happens
/// in the frontend.
///
/// Supports three plugin patterns:
/// - Inline: `&function(args){content};`
/// - Block multiline: `@function(args){{ content }}`
/// - Block singleline: `@function(args){content}`
///
/// Content within plugins is preserved as-is and can contain nested plugins
/// or other Wiki syntax that will be processed by the plugin at runtime.
///
/// # Arguments
///
/// * `html` - The HTML content to process
///
/// # Returns
///
/// HTML with plugin syntax converted to containers
///
/// # Examples
///
/// ```
/// use universal_markdown::extensions::plugins::apply_plugin_syntax;
///
/// // Block plugin
/// let input = "@toc(2){{ }}";
/// let output = apply_plugin_syntax(input);
/// assert!(output.contains("class=\"umd-plugin umd-plugin-toc\""));
/// // data-args is base64 encoded for security
/// assert!(output.contains("data-args=\""));
///
/// // Inline plugin
/// let input = "&highlight(yellow){important text};";
/// let output = apply_plugin_syntax(input);
/// assert!(output.contains("class=\"umd-plugin umd-plugin-highlight\""));
/// ```
pub fn apply_plugin_syntax(html: &str) -> String {
    let mut result = html.to_string();

    // Process block plugins (multiline) first - @function(args){{ content }}
    result = BLOCK_PLUGIN_MULTILINE
        .replace_all(&result, |caps: &regex::Captures| {
            let function = caps.get(1).map_or("", |m| m.as_str());
            let args = caps.get(2).map_or("", |m| m.as_str());
            let content = caps.get(3).map_or("", |m| m.as_str());

            let escaped_content = content.replace('<', "&lt;").replace('>', "&gt;");
            let json_args = args_to_json(args);
            let encoded_args = encode_args(&json_args);
            format!(
                "\n<div class=\"umd-plugin umd-plugin-{}\" data-args=\"{}\">{}\n</div>\n",
                function, encoded_args, escaped_content
            )
        })
        .to_string();

    // Process block plugins (singleline) - @function(args){content}
    result = BLOCK_PLUGIN_SINGLELINE
        .replace_all(&result, |caps: &regex::Captures| {
            let function = caps.get(1).map_or("", |m| m.as_str());
            let args = caps.get(2).map_or("", |m| m.as_str());
            let content = caps.get(3).map_or("", |m| m.as_str());

            let escaped_content = content.replace('<', "&lt;").replace('>', "&gt;");
            let json_args = args_to_json(args);
            let encoded_args = encode_args(&json_args);
            format!(
                "\n<div class=\"umd-plugin umd-plugin-{}\" data-args=\"{}\">{}\n</div>\n",
                function, encoded_args, escaped_content
            )
        })
        .to_string();

    // Process block plugins (args only, no content) - @function(args)
    result = BLOCK_PLUGIN_ARGSONLY
        .replace_all(&result, |caps: &regex::Captures| {
            let function = caps.get(1).map_or("", |m| m.as_str());
            let args = caps.get(2).map_or("", |m| m.as_str());

            let json_args = args_to_json(args);
            let encoded_args = encode_args(&json_args);
            format!(
                "\n<div class=\"umd-plugin umd-plugin-{}\" data-args=\"{}\" />\n",
                function, encoded_args
            )
        })
        .to_string();

    // Process block plugins (no args) - @function()
    result = BLOCK_PLUGIN_NOARGS
        .replace_all(&result, |caps: &regex::Captures| {
            let function = caps.get(1).map_or("", |m| m.as_str());
            let encoded_args = encode_args("[]");
            format!(
                "\n<div class=\"umd-plugin umd-plugin-{}\" data-args=\"{}\" />\n",
                function, encoded_args
            )
        })
        .to_string();

    // Process inline plugins - &function(args){content};
    result = INLINE_PLUGIN
        .replace_all(&result, |caps: &regex::Captures| {
            let function = caps.get(1).map_or("", |m| m.as_str());
            let args = caps.get(2).map_or("", |m| m.as_str());
            let content = caps.get(3).map_or("", |m| m.as_str());

            let escaped_content = content.replace('<', "&lt;").replace('>', "&gt;");
            let json_args = args_to_json(args);
            let encoded_args = encode_args(&json_args);
            format!(
                "<span class=\"umd-plugin umd-plugin-{}\" data-args=\"{}\">{}</span>",
                function, encoded_args, escaped_content
            )
        })
        .to_string();

    // Process inline plugins (args only) - &function(args);
    result = INLINE_PLUGIN_ARGSONLY
        .replace_all(&result, |caps: &regex::Captures| {
            let function = caps.get(1).map_or("", |m| m.as_str());
            let args = caps.get(2).map_or("", |m| m.as_str());

            let json_args = args_to_json(args);
            let encoded_args = encode_args(&json_args);
            format!(
                "<span class=\"umd-plugin umd-plugin-{}\" data-args=\"{}\" />",
                function, encoded_args
            )
        })
        .to_string();

    // Process inline plugins (no args) - &function;
    result = INLINE_PLUGIN_NOARGS
        .replace_all(&result, |caps: &regex::Captures| {
            let function = caps.get(1).map_or("", |m| m.as_str());
            let encoded_args = encode_args("[]");
            format!(
                "<span class=\"umd-plugin umd-plugin-{}\" data-args=\"{}\" />",
                function, encoded_args
            )
        })
        .to_string();

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_plugin() {
        let input = "@toc(2){{ }}";
        let output = apply_plugin_syntax(input);
        assert!(output.contains("class=\"umd-plugin umd-plugin-toc\""));
        assert!(output.contains("data-args=\"WyIyIl0=\""));
    }

    #[test]
    fn test_plugin_with_complex_args() {
        let input = "@calendar(2024,1,true){{ }}";
        let output = apply_plugin_syntax(input);
        assert!(output.contains("umd-plugin-calendar"));
        assert!(output.contains("data-args=\"WyIyMDI0IiwiMSIsInRydWUiXQ==\""));
    }

    #[test]
    fn test_plugin_no_args() {
        let input = "@timestamp(){{ }}";
        let output = apply_plugin_syntax(input);
        assert!(output.contains("umd-plugin-timestamp"));
        assert!(output.contains("data-args=\"W10=\""));
    }

    #[test]
    fn test_plugin_with_content() {
        let input = "@code(rust){{ fn main() {} }}";
        let output = apply_plugin_syntax(input);
        assert!(output.contains("umd-plugin-code"));
        assert!(output.contains("data-args=\"WyJydXN0Il0=\""));
        assert!(output.contains("fn main()"));
    }

    #[test]
    fn test_multiple_plugins() {
        let input = "@toc(2){{ }} and @timestamp(){{ }}";
        let output = apply_plugin_syntax(input);
        assert!(output.contains("umd-plugin-toc"));
        assert!(output.contains("umd-plugin-timestamp"));
    }

    #[test]
    fn test_no_plugin() {
        let input = "This is normal text with @mention but not a plugin";
        let output = apply_plugin_syntax(input);
        // @mention without parens should not match
        assert_eq!(output, input);
    }

    #[test]
    fn test_inline_plugin() {
        let input = "&highlight(yellow){important text};";
        let output = apply_plugin_syntax(input);
        assert!(output.contains("class=\"umd-plugin umd-plugin-highlight\""));
        assert!(output.contains("data-args=\"WyJ5ZWxsb3ciXQ==\""));
        assert!(output.contains("important text"));
        assert!(output.contains("<span"));
    }

    #[test]
    fn test_block_plugin_singleline() {
        let input = "@include(file.txt){default content}";
        let output = apply_plugin_syntax(input);
        assert!(output.contains("class=\"umd-plugin umd-plugin-include\""));
        assert!(output.contains("data-args=\"WyJmaWxlLnR4dCJd\""));
        assert!(output.contains("default content"));
    }

    #[test]
    fn test_nested_plugins() {
        let input = "&outer(arg1){text &inner(arg2){nested}; more};";
        let output = apply_plugin_syntax(input);
        assert!(output.contains("class=\"umd-plugin umd-plugin-outer\""));
        // Content should preserve the nested plugin syntax (& not escaped)
        assert!(output.contains("&inner"));
    }

    #[test]
    fn test_plugin_with_wiki_syntax() {
        let input = "@box(){{ **bold** and text }}";
        let output = apply_plugin_syntax(input);
        assert!(output.contains("class=\"umd-plugin umd-plugin-box\""));
        // Content should preserve wiki syntax for JS processing
        assert!(output.contains("**bold**"));
    }

    #[test]
    fn test_mixed_plugin_types() {
        let input = "@block(){{ content }} and &inline(arg){text}; mixed";
        let output = apply_plugin_syntax(input);
        assert!(output.contains("umd-plugin-block"));
        assert!(output.contains("umd-plugin-inline"));
    }

    // New tests for additional patterns
    #[test]
    fn test_block_plugin_no_args() {
        let input = "@toc()";
        let output = apply_plugin_syntax(input);
        assert!(output.contains("class=\"umd-plugin umd-plugin-toc\""));
        assert!(output.contains("data-args=\"W10=\""));
    }

    #[test]
    fn test_block_plugin_args_only() {
        let input = "@feed(https://example.com/feed.atom, 10)";
        let output = apply_plugin_syntax(input);
        assert!(output.contains("class=\"umd-plugin umd-plugin-feed\""));
        assert!(output.contains("data-args=\"WyJodHRwczovL2V4YW1wbGUuY29tL2ZlZWQuYXRvbSIsIjEwIl0=\""));
        assert!(output.contains("/>")); // Self-closing div
    }

    #[test]
    fn test_inline_plugin_args_only() {
        let input = "&icon(mdi-pencil);";
        let output = apply_plugin_syntax(input);
        assert!(output.contains("class=\"umd-plugin umd-plugin-icon\""));
        assert!(output.contains("data-args=\"WyJtZGktcGVuY2lsIl0=\""));
        assert!(output.contains("/>")); // Self-closing tag
    }

    #[test]
    fn test_inline_plugin_no_args() {
        let input = "&br;";
        let output = apply_plugin_syntax(input);
        assert!(output.contains("class=\"umd-plugin umd-plugin-br\""));
        assert!(output.contains("data-args=\"W10=\""));
        assert!(output.contains("/>")); // Self-closing tag
    }

    #[test]
    fn test_args_to_json_empty() {
        assert_eq!(args_to_json(""), "[]");
    }

    #[test]
    fn test_args_to_json_single() {
        assert_eq!(args_to_json("arg1"), "[\"arg1\"]");
    }

    #[test]
    fn test_args_to_json_multiple() {
        assert_eq!(
            args_to_json("arg1,arg2,arg3"),
            "[\"arg1\",\"arg2\",\"arg3\"]"
        );
    }

    #[test]
    fn test_args_to_json_with_spaces() {
        assert_eq!(
            args_to_json("arg1 , arg2 , arg3"),
            "[\"arg1\",\"arg2\",\"arg3\"]"
        );
    }

    #[test]
    fn test_args_to_json_url() {
        assert_eq!(
            args_to_json("https://example.com/feed.atom, 10"),
            "[\"https://example.com/feed.atom\",\"10\"]"
        );
    }
}
