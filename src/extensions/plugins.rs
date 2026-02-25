//! Plugin system for Universal Markdown
//!
//! Provides plugin syntax support:
//! - Inline plugins: &function(args){content};
//! - Block plugins (multiline): @function(args){{ content }}
//! - Block plugins (single line): @function(args){content}
//!
//! Note: This only parses plugin syntax and outputs <template> with <data> elements.
//! Actual plugin execution is handled by backend (Nuxt/Laravel) or frontend.
//! Content within plugins may contain nested plugins or other Wiki syntax.

use once_cell::sync::Lazy;
use regex::Regex;

/// Escape HTML special characters
///
/// # Arguments
///
/// * `input` - Text to escape
///
/// # Returns
///
/// HTML-escaped string
fn escape_html_text(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Parse comma-separated args into a vector
///
/// # Arguments
///
/// * `args` - Comma-separated argument string
///
/// # Returns
///
/// Vector of trimmed argument strings
fn parse_args(args: &str) -> Vec<String> {
    if args.trim().is_empty() {
        return vec![];
    }
    args.split(',').map(|s| s.trim().to_string()).collect()
}

/// Render args as <data> elements
///
/// # Arguments
///
/// * `args` - Comma-separated argument string
///
/// # Returns
///
/// HTML string with <data value="index">arg</data> elements
fn render_args_as_data(args: &str) -> String {
    parse_args(args)
        .iter()
        .enumerate()
        .map(|(i, arg)| format!("<data value=\"{}\">{}</data>", i, escape_html_text(arg)))
        .collect::<Vec<_>>()
        .join("")
}

// Standard plugins that output direct HTML instead of <template>
// @detail plugin for <details> element
static CLEAR_PLUGIN: Lazy<Regex> = Lazy::new(|| Regex::new(r"@clear\(\)").unwrap());

static DETAIL_PLUGIN: Lazy<Regex> = Lazy::new(|| {
    // Match @detail(summary) or @detail(summary, open){{ content }}
    Regex::new(r"@detail\(([^,)]+)(?:,\s*open)?\)\{\{([\s\S]*?)\}\}").unwrap()
});

static DETAIL_PLUGIN_OPEN: Lazy<Regex> = Lazy::new(|| {
    // Separate pattern to detect 'open' attribute
    Regex::new(r"@detail\([^,)]+,\s*open\)").unwrap()
});

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
    // Function name must start with a letter to avoid conflicts with HTML entities
    Regex::new(r"&([a-zA-Z]\w*);").unwrap()
});

// Common HTML entities that should NOT be treated as plugins
static HTML_ENTITIES: Lazy<std::collections::HashSet<&'static str>> = Lazy::new(|| {
    [
        "lt", "gt", "amp", "nbsp", "quot", "apos", "ndash", "mdash", "hellip", "copy", "reg",
        "trade", "times", "divide", "plusmn", "le", "ge", "ne", "asymp", "equiv", "forall",
        "exist", "empty", "nabla", "isin", "notin", "ni", "prod", "sum", "minus", "lowast",
        "radic", "prop", "infin", "ang", "and", "or", "cap", "cup", "int", "there4", "sim", "cong",
        "sub", "sup", "nsub", "sube", "supe", "oplus", "otimes", "perp", "sdot", "lceil", "rceil",
        "lfloor", "rfloor", "lang", "rang", "loz", "spades", "clubs", "hearts", "diams", "alpha",
        "beta", "gamma", "delta", "epsilon", "zeta", "eta", "theta", "iota", "kappa", "lambda",
        "mu", "nu", "xi", "omicron", "pi", "rho", "sigma", "tau", "upsilon", "phi", "chi", "psi",
        "omega", "Iuml", "iuml", "Uuml", "uuml", "Auml", "auml", "Ouml", "ouml", "Euml", "euml",
        "Aring", "aring", "AElig", "aelig", "Ccedil", "ccedil", "Eth", "eth", "Ntilde", "ntilde",
        "Oslash", "oslash", "Thorn", "thorn", "szlig", "yuml", "Agrave", "agrave", "Aacute",
        "aacute", "Acirc", "acirc", "Atilde", "atilde", "Egrave", "egrave", "Eacute", "eacute",
        "Ecirc", "ecirc", "Igrave", "igrave", "Iacute", "iacute", "Icirc", "icirc", "Ograve",
        "ograve", "Oacute", "oacute", "Ocirc", "ocirc", "Otilde", "otilde", "Ugrave", "ugrave",
        "Uacute", "uacute", "Ucirc", "ucirc", "Yacute", "yacute", "cent", "pound", "curren", "yen",
        "brvbar", "sect", "uml", "ordf", "laquo", "not", "shy", "macr", "deg", "sup2", "sup3",
        "acute", "micro", "para", "middot", "cedil", "sup1", "ordm", "raquo", "frac14", "frac12",
        "frac34", "iquest", "ensp", "emsp", "thinsp", "zwnj", "zwj", "lrm", "rlm",
    ]
    .iter()
    .copied()
    .collect()
});

/// Apply plugin syntax transformation
///
/// Converts plugin syntax to <template> elements with <data> children.
/// The parser only detects and preserves plugin metadata; actual execution happens
/// on the backend (Nuxt/Laravel) or frontend.
///
/// Supports multiple plugin patterns:
/// - Inline: `&function(args){content};`
/// - Block multiline: `@function(args){{ content }}`
/// - Block singleline: `@function(args){content}`
/// - Args only: `@function(args)` or `&function(args);`
/// - No args: `@function()` or `&function;`
///
/// Content within plugins is escaped and can contain nested plugins
/// or other Wiki syntax that will be processed by the plugin at runtime.
///
/// # Plugin-specific behavior
///
/// **Table plugin (`@table`)**: Only the first table element found within `{{}}`
/// will be processed with the specified Bootstrap classes. Multiple tables or
/// nested `@table` plugins are not recommended and may cause unexpected behavior.
///
/// # Arguments
///
/// * `html` - The HTML content to process
///
/// # Returns
///
/// HTML with plugin syntax converted to <template> containers
///
/// # Examples
///
/// ```
/// use umd::extensions::plugins::apply_plugin_syntax;
///
/// // Block plugin
/// let input = "@toc(2){{ }}";
/// let output = apply_plugin_syntax(input);
/// assert!(output.contains("class=\"umd-plugin umd-plugin-toc\""));
/// assert!(output.contains("<data value=\"0\">2</data>"));
///
/// // Inline plugin
/// let input = "&highlight(yellow){important text};";
/// let output = apply_plugin_syntax(input);
/// assert!(output.contains("class=\"umd-plugin umd-plugin-highlight\""));
/// assert!(output.contains("<data value=\"0\">yellow</data>"));
/// assert!(output.contains("important text"));
/// ```
pub fn apply_plugin_syntax(html: &str) -> String {
    let mut result = html.to_string();

    // Process standard plugins first - @clear() outputs a clearfix block
    result = CLEAR_PLUGIN
        .replace_all(&result, "\n<div class=\"clearfix\"></div>\n")
        .to_string();

    // Process standard plugins first - @detail(summary[, open]){{ content }}
    // This outputs direct HTML <details> instead of <template>
    result = DETAIL_PLUGIN
        .replace_all(&result, |caps: &regex::Captures| {
            let summary = caps.get(1).map_or("", |m| m.as_str().trim());
            let content = caps.get(2).map_or("", |m| m.as_str().trim());

            // Check if 'open' attribute is present in the full match
            let full_match = caps.get(0).map_or("", |m| m.as_str());
            let is_open = DETAIL_PLUGIN_OPEN.is_match(full_match);

            let open_attr = if is_open { " open" } else { "" };

            format!(
                "\n<details{}>\n  <summary>{}</summary>\n  {}\n</details>\n",
                open_attr, summary, content
            )
        })
        .to_string();

    // Process block plugins (multiline) first - @function(args){{ content }}
    result = BLOCK_PLUGIN_MULTILINE
        .replace_all(&result, |caps: &regex::Captures| {
            let function = caps.get(1).map_or("", |m| m.as_str());
            let args = caps.get(2).map_or("", |m| m.as_str());
            let content = caps.get(3).map_or("", |m| m.as_str());

            let args_html = render_args_as_data(args);
            let escaped_content = escape_html_text(content);

            if escaped_content.is_empty() {
                format!(
                    "\n<template class=\"umd-plugin umd-plugin-{}\">{}</template>\n",
                    function, args_html
                )
            } else {
                format!(
                    "\n<template class=\"umd-plugin umd-plugin-{}\">{}{}</template>\n",
                    function, args_html, escaped_content
                )
            }
        })
        .to_string();

    // Process block plugins (singleline) - @function(args){content}
    result = BLOCK_PLUGIN_SINGLELINE
        .replace_all(&result, |caps: &regex::Captures| {
            let function = caps.get(1).map_or("", |m| m.as_str());
            let args = caps.get(2).map_or("", |m| m.as_str());
            let content = caps.get(3).map_or("", |m| m.as_str());

            let args_html = render_args_as_data(args);
            let escaped_content = escape_html_text(content);

            if escaped_content.is_empty() {
                format!(
                    "\n<template class=\"umd-plugin umd-plugin-{}\">{}</template>\n",
                    function, args_html
                )
            } else {
                format!(
                    "\n<template class=\"umd-plugin umd-plugin-{}\">{}{}</template>\n",
                    function, args_html, escaped_content
                )
            }
        })
        .to_string();

    // Process block plugins (args only, no content) - @function(args)
    result = BLOCK_PLUGIN_ARGSONLY
        .replace_all(&result, |caps: &regex::Captures| {
            let function = caps.get(1).map_or("", |m| m.as_str());
            let args = caps.get(2).map_or("", |m| m.as_str());

            let args_html = render_args_as_data(args);
            format!(
                "\n<template class=\"umd-plugin umd-plugin-{}\">{}</template>\n",
                function, args_html
            )
        })
        .to_string();

    // Process block plugins (no args) - @function()
    result = BLOCK_PLUGIN_NOARGS
        .replace_all(&result, |caps: &regex::Captures| {
            let function = caps.get(1).map_or("", |m| m.as_str());
            format!(
                "\n<template class=\"umd-plugin umd-plugin-{}\"></template>\n",
                function
            )
        })
        .to_string();

    // Process inline plugins - &function(args){content};
    result = INLINE_PLUGIN
        .replace_all(&result, |caps: &regex::Captures| {
            let function = caps.get(1).map_or("", |m| m.as_str());
            let args = caps.get(2).map_or("", |m| m.as_str());
            let content = caps.get(3).map_or("", |m| m.as_str());

            let args_html = render_args_as_data(args);
            let escaped_content = escape_html_text(content);

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

    // Process inline plugins (args only) - &function(args);
    result = INLINE_PLUGIN_ARGSONLY
        .replace_all(&result, |caps: &regex::Captures| {
            let function = caps.get(1).map_or("", |m| m.as_str());
            let args = caps.get(2).map_or("", |m| m.as_str());

            let args_html = render_args_as_data(args);
            format!(
                "<template class=\"umd-plugin umd-plugin-{}\">{}</template>",
                function, args_html
            )
        })
        .to_string();

    // Process inline plugins (no args) - &function;
    result = INLINE_PLUGIN_NOARGS
        .replace_all(&result, |caps: &regex::Captures| {
            let function = caps.get(1).map_or("", |m| m.as_str());

            // Skip HTML entities
            if HTML_ENTITIES.contains(function) {
                return caps[0].to_string(); // Return original match unchanged
            }

            format!(
                "<template class=\"umd-plugin umd-plugin-{}\"></template>",
                function
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
        assert!(output.contains("<data value=\"0\">2</data>"));
    }

    #[test]
    fn test_plugin_with_complex_args() {
        let input = "@calendar(2024,1,true){{ }}";
        let output = apply_plugin_syntax(input);
        assert!(output.contains("umd-plugin-calendar"));
        assert!(output.contains("<data value=\"0\">2024</data>"));
        assert!(output.contains("<data value=\"1\">1</data>"));
        assert!(output.contains("<data value=\"2\">true</data>"));
    }

    #[test]
    fn test_plugin_no_args() {
        let input = "@timestamp(){{ }}";
        let output = apply_plugin_syntax(input);
        assert!(output.contains("umd-plugin-timestamp"));
        assert!(!output.contains("<data"));
    }

    #[test]
    fn test_plugin_with_content() {
        let input = "@code(rust){{ fn main() {} }}";
        let output = apply_plugin_syntax(input);
        assert!(output.contains("umd-plugin-code"));
        assert!(output.contains("<data value=\"0\">rust</data>"));
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
        assert!(output.contains("<data value=\"0\">yellow</data>"));
        assert!(output.contains("important text"));
        assert!(output.contains("<template"));
    }

    #[test]
    fn test_block_plugin_singleline() {
        let input = "@include(file.txt){default content}";
        let output = apply_plugin_syntax(input);
        assert!(output.contains("class=\"umd-plugin umd-plugin-include\""));
        assert!(output.contains("<data value=\"0\">file.txt</data>"));
        assert!(output.contains("default content"));
    }

    #[test]
    fn test_nested_plugins() {
        let input = "&outer(arg1){text &inner(arg2){nested}; more};";
        let output = apply_plugin_syntax(input);
        println!("Nested output: {}", output);
        assert!(output.contains("class=\"umd-plugin umd-plugin-outer\""));
        // Content should preserve the nested plugin syntax (escaped)
        // & is escaped to &amp; in the content
        assert!(output.contains("&amp;"));
    }

    #[test]
    fn test_plugin_with_wiki_syntax() {
        let input = "@box(){{ **bold** and text }}";
        let output = apply_plugin_syntax(input);
        assert!(output.contains("class=\"umd-plugin umd-plugin-box\""));
        // Content should preserve wiki syntax (escaped for backend processing)
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
        assert!(!output.contains("<data"));
    }

    #[test]
    fn test_detail_plugin_basic() {
        let input = "@detail(Click to expand){{ Hidden content }}";
        let output = apply_plugin_syntax(input);
        assert!(output.contains("<details>"));
        assert!(output.contains("<summary>Click to expand</summary>"));
        assert!(output.contains("Hidden content"));
        assert!(output.contains("</details>"));
        assert!(!output.contains("open")); // Should not have open attribute
    }

    #[test]
    fn test_detail_plugin_with_open() {
        let input = "@detail(Already visible, open){{ This is shown by default }}";
        let output = apply_plugin_syntax(input);
        assert!(output.contains("<details open>"));
        assert!(output.contains("<summary>Already visible</summary>"));
        assert!(output.contains("This is shown by default"));
    }

    #[test]
    fn test_block_plugin_args_only() {
        let input = "@feed(https://example.com/feed.atom, 10)";
        let output = apply_plugin_syntax(input);
        assert!(output.contains("class=\"umd-plugin umd-plugin-feed\""));
        assert!(output.contains("<data value=\"0\">https://example.com/feed.atom</data>"));
        assert!(output.contains("<data value=\"1\">10</data>"));
    }

    #[test]
    fn test_inline_plugin_args_only() {
        let input = "&icon(mdi-pencil);";
        let output = apply_plugin_syntax(input);
        assert!(output.contains("class=\"umd-plugin umd-plugin-icon\""));
        assert!(output.contains("<data value=\"0\">mdi-pencil</data>"));
    }

    #[test]
    fn test_inline_plugin_no_args() {
        let input = "&br;";
        let output = apply_plugin_syntax(input);
        assert!(output.contains("class=\"umd-plugin umd-plugin-br\""));
        assert!(!output.contains("<data"));
    }

    #[test]
    fn test_html_escaping_in_args() {
        let input = "@test(<script>alert('xss')</script>){{ }}";
        let output = apply_plugin_syntax(input);
        println!("Escaped args output: {}", output);
        // Args are escaped in <data> elements
        assert!(output.contains("&lt;"));
        assert!(output.contains("&gt;"));
    }

    #[test]
    fn test_html_escaping_in_content() {
        let input = "&test(arg){<b>content</b>};";
        let output = apply_plugin_syntax(input);
        println!("Escaped content output: {}", output);
        // Content is escaped
        assert!(output.contains("&lt;"));
        assert!(output.contains("&gt;"));
    }
}
