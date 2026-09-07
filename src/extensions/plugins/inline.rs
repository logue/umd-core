//! Inline plugin notation: `&function(args){content};` (and the
//! args-only/no-args/no-args-content variants).
//!
//! Processing happens in two passes:
//!
//! 1. **Protect + restore** (`protect_inline_plugins` / `restore_markers`):
//!    the primary path. `protect_inline_plugins` runs before Markdown
//!    parsing and turns each call into a `{{INLINE_PLUGIN...}}`-style
//!    marker so Markdown never sees it; `restore_markers` runs during
//!    `conflict_resolver::postprocess_conflicts_with_options` and turns
//!    each marker into real HTML via `convert_standard_inline_plugin_to_html`
//!    (and its `argsonly`/`noargs` siblings) for recognized ("standard")
//!    function names, falling back to a generic
//!    `<template class="umd-plugin umd-plugin-inline umd-plugin-*">` for
//!    anything else.
//!
//! 2. **Second-pass sweep** (`prepare` + `expand`): the marker regex used by
//!    `protect_inline_plugins` allows one level of `{...}` nesting in a
//!    call's content, so a standard plugin nested inside *another* standard
//!    plugin's content (e.g. `&color(blue){outer &abbr(t){d}; end};`)
//!    survives the first pass as literal, unexpanded text embedded in the
//!    outer HTML. `expand` is that second sweep — it re-runs the standard
//!    plugin regexes once more over the already-restored HTML to catch
//!    exactly that leftover case. `prepare` handles the two steps that must
//!    run first: decoding the `&amp;color(` etc. entities comrak produces
//!    (so the sweep's regexes can see a literal `&`), and (if a nesting
//!    limit is configured) neutralizing calls beyond that limit.
//!
//! Because both passes must agree on what a given call produces, both call
//! into `decoration_values::{map_color_value_with_options, map_font_size_value}`
//! (also shared with `block_decoration.rs`'s `COLOR()`/`SIZE()`) rather than
//! each declaring its own copy of the palette/keyword logic — don't
//! reintroduce a local copy here.

use base64::{Engine as _, engine::general_purpose};
use once_cell::sync::Lazy;
use regex::{Captures, Regex};
use std::collections::HashSet;

use super::{escape_html_text, render_args_as_data, render_math_html, render_popover_html};
use crate::extensions::decoration_values::{map_color_value_with_options, map_font_size_value};

/// HTML entities that should NOT be treated as plugins
fn html_entities() -> HashSet<&'static str> {
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
}

/// Protect inline plugin syntax by converting to markers
///
/// Converts various inline plugin patterns into safe markers:
/// - `&function{content};` → marker with content
/// - `&function(args){content};` → marker with args and content
/// - `&function(args);` → marker with args
/// - `&function;` → marker (excluding HTML entities)
pub fn protect_inline_plugins(input: &str) -> String {
    let mut result = input.to_string();

    // Protect inline plugins with content but no args: &function{content};
    let inline_plugin_noargs_content = Regex::new(r"&(\w+)\{((?:[^{}]|\{[^}]*\})*)\};").unwrap();
    result = inline_plugin_noargs_content
        .replace_all(&result, |caps: &regex::Captures| {
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
    let inline_plugin = Regex::new(r"&(\w+)\(([^)]*)\)\{((?:[^{}]|\{[^}]*\})*)\};").unwrap();
    result = inline_plugin
        .replace_all(&result, |caps: &regex::Captures| {
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
    // Function name must start with a letter to avoid conflicts with HTML entities
    let inline_plugin_noargs = Regex::new(r"&([a-zA-Z]\w*);").unwrap();
    let entities = html_entities();

    result = inline_plugin_noargs
        .replace_all(&result, |caps: &regex::Captures| {
            let function = &caps[1];

            // Skip HTML entities
            if entities.contains(function) {
                return caps[0].to_string();
            }

            format!(
                "{{{{INLINE_PLUGIN_NOARGS:{}:INLINE_PLUGIN_NOARGS}}}}",
                function
            )
        })
        .to_string();

    result
}

/// Dispatch a `&function(args){content};` call to its built-in ("standard")
/// inline plugin implementation, producing real semantic HTML directly.
///
/// This is the inline counterpart of the block-level standard plugins
/// (`@table`/`@math`/`@popover`/`@clear`/`@detail`, see `plugins::block`) —
/// a function name recognized here bypasses the generic
/// `<template class="umd-plugin umd-plugin-inline umd-plugin-{name}">`
/// fallback that unrecognized/host-defined plugin names get. Returns `None` for anything not in this
/// list, letting the caller fall back to the generic template.
fn convert_standard_inline_plugin_to_html(
    function: &str,
    args: &str,
    content: &str,
    allow_hex_colors: bool,
    allow_custom_font_size: bool,
) -> Option<String> {
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
        "spoiler" => {
            // &spoiler{text}; → <span class="umd-spoiler" role="button" ...>text</span>
            Some(format!(
                r#"<span class="umd-spoiler" role="button" tabindex="0" aria-expanded="false">{}</span>"#,
                content
            ))
        }

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
            // &color(fg,bg){text}; with Bootstrap support
            let parts: Vec<&str> = args.split(',').collect();
            let fg = parts.get(0).map_or("", |m| m.trim());
            let bg = parts.get(1).map_or("", |m| m.trim());

            let mut classes = Vec::new();
            let mut styles = Vec::new();

            if !fg.is_empty() && fg != "inherit" {
                if let Some((is_class, value)) =
                    map_color_value_with_options(fg, false, allow_hex_colors)
                {
                    if is_class {
                        classes.push(value);
                    } else {
                        styles.push(format!("color: {}", value));
                    }
                }
            }

            if !bg.is_empty() && bg != "inherit" {
                if let Some((is_class, value)) =
                    map_color_value_with_options(bg, true, allow_hex_colors)
                {
                    if is_class {
                        classes.push(value);
                    } else {
                        styles.push(format!("background-color: {}", value));
                    }
                }
            }

            if classes.is_empty() && styles.is_empty() {
                Some(content.to_string())
            } else {
                let mut attrs = Vec::new();
                if !classes.is_empty() {
                    attrs.push(format!("class=\"{}\"", classes.join(" ")));
                }
                if !styles.is_empty() {
                    attrs.push(format!("style=\"{}\"", styles.join("; ")));
                }
                Some(format!("<span {}>{}</span>", attrs.join(" "), content))
            }
        }
        "size" => {
            // &size(xs|sm|lg|xl){text}; → <span class="umd-text-size-*">
            // (or, with allow_custom_font_size, &size(value){text}; → inline font-size)
            match map_font_size_value(args, allow_custom_font_size) {
                Some((true, class)) => {
                    Some(format!("<span class=\"{}\">{}</span>", class, content))
                }
                Some((false, value)) => Some(format!(
                    "<span style=\"font-size: {}\">{}</span>",
                    value, content
                )),
                None => Some(content.to_string()),
            }
        }
        _ => None,
    }
}

/// Standard inline plugin dispatch for the args-only form: `&function(args);`
fn convert_standard_inline_plugin_argsonly_to_html(function: &str, args: &str) -> Option<String> {
    match function {
        "sup" => Some(format!("<sup>{}</sup>", args)),
        "sub" => Some(format!("<sub>{}</sub>", args)),
        "math" => render_math_html(args, false),
        "spoiler" => {
            // &spoiler(text); → <span class="umd-spoiler" role="button" ...>text</span>
            Some(format!(
                r#"<span class="umd-spoiler" role="button" tabindex="0" aria-expanded="false">{}</span>"#,
                args
            ))
        }
        _ => None,
    }
}

/// Standard inline plugin dispatch for the no-args form: `&function;`
fn convert_standard_inline_plugin_noargs_to_html(function: &str) -> Option<String> {
    match function {
        "wbr" => Some("<wbr />".to_string()),
        "br" => Some("<br />".to_string()),
        _ => None,
    }
}

/// Restore `{{INLINE_PLUGIN...}}`-style markers left by `protect_inline_plugins`.
///
/// Called from `conflict_resolver::postprocess_conflicts_with_options` at
/// the same point the three marker-restoration regex blocks used to run
/// inline — see that function for the surrounding restore order.
pub(crate) fn restore_markers(
    html: &str,
    allow_hex_colors: bool,
    allow_custom_font_size: bool,
) -> String {
    let mut result = html.to_string();

    // Restore inline plugins
    let inline_plugin_marker =
        Regex::new(r"\{\{INLINE_PLUGIN:(\w+):([\s\S]*?):([\s\S]*?):INLINE_PLUGIN\}\}").unwrap();
    result = inline_plugin_marker
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
                if let Some(mathml) = render_math_html(formula, false) {
                    return mathml;
                }
            }

            if function == "popover" {
                return render_popover_html(args, &content);
            }

            // Try to convert as inline decoration function
            if let Some(html) = convert_standard_inline_plugin_to_html(
                function,
                args,
                &content,
                allow_hex_colors,
                allow_custom_font_size,
            ) {
                return html;
            }

            // Otherwise, convert to plugin <template>
            let args_html = render_args_as_data(args);
            let escaped_content = escape_html_text(&content);

            if escaped_content.is_empty() {
                format!(
                    "<template class=\"umd-plugin umd-plugin-inline umd-plugin-{}\">{}</template>",
                    function, args_html
                )
            } else {
                format!(
                    "<template class=\"umd-plugin umd-plugin-inline umd-plugin-{}\">{}{}</template>",
                    function, args_html, escaped_content
                )
            }
        })
        .to_string();

    // Restore inline plugins (args only)
    let inline_plugin_argsonly_marker =
        Regex::new(r"\{\{INLINE_PLUGIN_ARGSONLY:(\w+):([\s\S]*?):INLINE_PLUGIN_ARGSONLY\}\}")
            .unwrap();
    result = inline_plugin_argsonly_marker
        .replace_all(&result, |caps: &Captures| {
            let function = &caps[1];
            let args = &caps[2];

            // Try to convert as inline decoration function
            if let Some(html) = convert_standard_inline_plugin_argsonly_to_html(function, args) {
                return html;
            }

            // Otherwise, convert to plugin <template>
            let args_html = render_args_as_data(args);
            format!(
                "<template class=\"umd-plugin umd-plugin-inline umd-plugin-{}\">{}</template>",
                function, args_html
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
            if let Some(html) = convert_standard_inline_plugin_noargs_to_html(function) {
                return html;
            }

            // Otherwise, convert to plugin <template>
            format!(
                "<template class=\"umd-plugin umd-plugin-inline umd-plugin-{}\"></template>",
                function
            )
        })
        .to_string();

    result
}

static INLINE_COLOR: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"&color\(([^,)]*?)(?:,([^)]*?))?\)\{([^}]+?)\};").unwrap());

static INLINE_SIZE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"&size\(([^)]+?)\)\{([^}]+?)\};").unwrap());

static INLINE_SUP: Lazy<Regex> = Lazy::new(|| Regex::new(r"&sup\(([^)]+?)\);").unwrap());

static INLINE_SUB: Lazy<Regex> = Lazy::new(|| Regex::new(r"&sub\(([^)]+?)\);").unwrap());

static INLINE_LANG: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"&lang\(([^)]+?)\)\{([^}]+?)\};").unwrap());

static INLINE_ABBR: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"&abbr\(([^)]+?)\)\{([^}]+?)\};").unwrap());

static INLINE_RUBY: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"&ruby\(([^)]+?)\)\{([^}]+?)\};").unwrap());

// Semantic HTML elements - simple wrapper tags
static INLINE_DFN: Lazy<Regex> = Lazy::new(|| Regex::new(r"&dfn\(([^)]+?)\);").unwrap());
static INLINE_KBD: Lazy<Regex> = Lazy::new(|| Regex::new(r"&kbd\(([^)]+?)\);").unwrap());
static INLINE_SAMP: Lazy<Regex> = Lazy::new(|| Regex::new(r"&samp\(([^)]+?)\);").unwrap());
static INLINE_VAR: Lazy<Regex> = Lazy::new(|| Regex::new(r"&var\(([^)]+?)\);").unwrap());
static INLINE_CITE: Lazy<Regex> = Lazy::new(|| Regex::new(r"&cite\(([^)]+?)\);").unwrap());
static INLINE_Q: Lazy<Regex> = Lazy::new(|| Regex::new(r"&q\(([^)]+?)\);").unwrap());
static INLINE_SMALL: Lazy<Regex> = Lazy::new(|| Regex::new(r"&small\(([^)]+?)\);").unwrap());

// Elements with attributes
static INLINE_TIME: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"&time\(([^)]+?)\)\{([^}]+?)\};").unwrap());
static INLINE_DATA: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"&data\(([^)]+?)\)\{([^}]+?)\};").unwrap());

// Bidirectional text
static INLINE_BDI: Lazy<Regex> = Lazy::new(|| Regex::new(r"&bdi\(([^)]+?)\);").unwrap());
static INLINE_BDO: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"&bdo\(([^)]+?)\)\{([^}]+?)\};").unwrap());

// Word break opportunity (self-closing)
static INLINE_WBR: Lazy<Regex> = Lazy::new(|| Regex::new(r"&wbr;").unwrap());

// Manual line break (self-closing) - mainly for table cells where trailing spaces don't work
static INLINE_BR: Lazy<Regex> = Lazy::new(|| Regex::new(r"&br;").unwrap());

/// Regex for the standard `&spoiler()` plugin, second pass only (see module
/// docs): &spoiler(text); or &spoiler{text};
static INLINE_SPOILER: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"&spoiler(?:\(([^)]+?)\)|\{([^}]+?)\});").unwrap());

/// First half of the second-pass sweep: decode the `&amp;color(` etc.
/// entities comrak produces, then (if `max_inline_nesting` is set) neutralize
/// calls beyond that limit. Must run before `expand`. `inline::notation::apply`
/// (plain `%%`/`||` notation) runs in between the two in the real pipeline —
/// see `extensions::apply_extensions_with_headers`.
pub(crate) fn prepare(html: &str, max_inline_nesting: Option<usize>) -> String {
    let mut result = html.to_string();

    // Decode HTML entities for UMD inline syntax
    // Comrak escapes & to &amp;, which prevents our regexes from matching
    // We need to convert &amp; back to & for UMD syntax only
    result = result.replace("&amp;color(", "&color(");
    result = result.replace("&amp;size(", "&size(");
    result = result.replace("&amp;sup(", "&sup(");
    result = result.replace("&amp;sub(", "&sub(");
    result = result.replace("&amp;lang(", "&lang(");
    result = result.replace("&amp;abbr(", "&abbr(");
    result = result.replace("&amp;ruby(", "&ruby(");
    result = result.replace("&amp;spoiler(", "&spoiler(");
    result = result.replace("&amp;spoiler{", "&spoiler{");
    result = result.replace("&amp;dfn(", "&dfn(");
    result = result.replace("&amp;kbd(", "&kbd(");
    result = result.replace("&amp;samp(", "&samp(");
    result = result.replace("&amp;var(", "&var(");
    result = result.replace("&amp;cite(", "&cite(");
    result = result.replace("&amp;q(", "&q(");
    result = result.replace("&amp;small(", "&small(");
    result = result.replace("&amp;time(", "&time(");
    result = result.replace("&amp;data(", "&data(");
    result = result.replace("&amp;bdi(", "&bdi(");
    result = result.replace("&amp;bdo(", "&bdo(");
    result = result.replace("&amp;wbr", "&wbr");
    result = result.replace("&amp;br", "&br");

    if let Some(limit) = max_inline_nesting.filter(|limit| *limit > 0) {
        if inline_decoration_nesting_depth(&result) > limit {
            result = neutralize_over_limit_inline_decorations(&result, limit);
        }
    }

    result
}

/// Second half of the second-pass sweep: re-expand any standard inline
/// plugin calls that survived the primary `restore_markers` pass as literal
/// text (because they were nested inside another standard plugin's marker
/// content). See module docs.
pub(crate) fn expand(html: &str, allow_hex_colors: bool, allow_custom_font_size: bool) -> String {
    let mut result = html.to_string();

    // Second pass for &spoiler(text); or &spoiler{text}; (see module docs)
    result = INLINE_SPOILER
        .replace_all(&result, |caps: &regex::Captures| {
            let text = caps.get(1).or_else(|| caps.get(2)).map_or("", |m| m.as_str());
            format!(r#"<span class="umd-spoiler" role="button" tabindex="0" aria-expanded="false">{}</span>"#, text)
        })
        .to_string();

    // Apply &color(fg,bg){text}; with Bootstrap support
    result = INLINE_COLOR
        .replace_all(&result, |caps: &regex::Captures| {
            let fg = caps.get(1).map_or("", |m| m.as_str().trim());
            let bg = caps.get(2).map_or("", |m| m.as_str().trim());
            let text = caps.get(3).map_or("", |m| m.as_str());

            let mut classes = Vec::new();
            let mut styles = Vec::new();

            if !fg.is_empty() && fg != "inherit" {
                if let Some((is_class, value)) =
                    map_color_value_with_options(fg, false, allow_hex_colors)
                {
                    if is_class {
                        classes.push(value);
                    } else {
                        styles.push(format!("color: {}", value));
                    }
                }
            }

            if !bg.is_empty() && bg != "inherit" {
                if let Some((is_class, value)) =
                    map_color_value_with_options(bg, true, allow_hex_colors)
                {
                    if is_class {
                        classes.push(value);
                    } else {
                        styles.push(format!("background-color: {}", value));
                    }
                }
            }

            if classes.is_empty() && styles.is_empty() {
                text.to_string()
            } else {
                let mut attrs = Vec::new();
                if !classes.is_empty() {
                    attrs.push(format!("class=\"{}\"", classes.join(" ")));
                }
                if !styles.is_empty() {
                    attrs.push(format!("style=\"{}\"", styles.join("; ")));
                }
                format!("<span {}>{}</span>", attrs.join(" "), text)
            }
        })
        .to_string();

    // Apply &size(xs|sm|lg|xl){text}; (or an arbitrary value, opt-in only)
    result = INLINE_SIZE
        .replace_all(&result, |caps: &regex::Captures| {
            let size = caps.get(1).map_or("", |m| m.as_str());
            let text = caps.get(2).map_or("", |m| m.as_str());

            match map_font_size_value(size, allow_custom_font_size) {
                Some((true, class)) => format!("<span class=\"{}\">{}</span>", class, text),
                Some((false, value)) => {
                    format!("<span style=\"font-size: {}\">{}</span>", value, text)
                }
                None => text.to_string(),
            }
        })
        .to_string();

    // Apply &sup(text);
    result = INLINE_SUP
        .replace_all(&result, "<sup>$1</sup>;")
        .to_string();

    // Apply &sub(text);
    result = INLINE_SUB
        .replace_all(&result, "<sub>$1</sub>;")
        .to_string();

    // Apply &lang(locale){text};
    result = INLINE_LANG
        .replace_all(&result, "<span lang=\"$1\">$2</span>;")
        .to_string();

    // Apply &abbr(text){description};
    result = INLINE_ABBR
        .replace_all(&result, "<abbr title=\"$2\">$1</abbr>;")
        .to_string();

    // Apply &ruby(reading){text};
    result = INLINE_RUBY
        .replace_all(&result, "<ruby>$2<rp>(</rp><rt>$1</rt><rp>)</rp></ruby>;")
        .to_string();

    // Semantic HTML elements - simple wrappers
    result = INLINE_DFN
        .replace_all(&result, "<dfn>$1</dfn>;")
        .to_string();
    result = INLINE_KBD
        .replace_all(&result, "<kbd>$1</kbd>;")
        .to_string();
    result = INLINE_SAMP
        .replace_all(&result, "<samp>$1</samp>;")
        .to_string();
    result = INLINE_VAR
        .replace_all(&result, "<var>$1</var>;")
        .to_string();
    result = INLINE_CITE
        .replace_all(&result, "<cite>$1</cite>;")
        .to_string();
    result = INLINE_Q.replace_all(&result, "<q>$1</q>;").to_string();
    result = INLINE_SMALL
        .replace_all(&result, "<small>$1</small>;")
        .to_string();

    // Elements with attributes
    result = INLINE_TIME
        .replace_all(&result, "<time datetime=\"$1\">$2</time>;")
        .to_string();
    result = INLINE_DATA
        .replace_all(&result, "<data value=\"$1\">$2</data>;")
        .to_string();

    // Bidirectional text
    result = INLINE_BDI
        .replace_all(&result, "<bdi>$1</bdi>;")
        .to_string();
    result = INLINE_BDO
        .replace_all(&result, "<bdo dir=\"$1\">$2</bdo>;")
        .to_string();

    // Word break opportunity
    result = INLINE_WBR.replace_all(&result, "<wbr />").to_string();

    // Manual line break (mainly for table cells)
    result = INLINE_BR.replace_all(&result, "<br />").to_string();

    result
}

fn inline_decoration_nesting_depth(input: &str) -> usize {
    let bytes = input.as_bytes();
    let mut i = 0usize;
    let mut stack: Vec<bool> = Vec::new();
    let mut limited_depth = 0usize;
    let mut max_limited_depth = 0usize;

    while i < bytes.len() {
        if let Some((name, next_index)) = parse_inline_block_start(input, i) {
            let limited = is_limited_inline_name(name);
            stack.push(limited);
            if limited {
                limited_depth += 1;
                max_limited_depth = max_limited_depth.max(limited_depth);
            }
            i = next_index;
            continue;
        }

        if i + 1 < bytes.len() && bytes[i] == b'}' && bytes[i + 1] == b';' && !stack.is_empty() {
            if stack.pop().unwrap_or(false) {
                limited_depth = limited_depth.saturating_sub(1);
            }
            i += 2;
            continue;
        }

        i += 1;
    }

    max_limited_depth
}

fn neutralize_over_limit_inline_decorations(input: &str, max_inline_nesting: usize) -> String {
    let bytes = input.as_bytes();
    let mut i = 0usize;
    let mut stack: Vec<bool> = Vec::new();
    let mut limited_depth = 0usize;
    let mut output = String::with_capacity(input.len() + 16);

    while i < bytes.len() {
        if let Some((name, next_index)) = parse_inline_block_start(input, i) {
            let limited = is_limited_inline_name(name);
            let over_limit = limited && limited_depth >= max_inline_nesting;

            if over_limit {
                if let Some(end_index) = find_inline_block_end(input, i) {
                    let escaped = html_escape::encode_safe(&input[i..end_index])
                        .replace('{', "&#123;")
                        .replace('}', "&#125;");
                    output.push_str(r#"<span class="umd-error-deep-recursive">"#);
                    output.push_str(&escaped);
                    output.push_str("</span>");
                    i = end_index;
                    continue;
                }

                let escaped = html_escape::encode_safe("&");
                output.push_str(escaped.as_ref());
                i += 1;
                continue;
            }

            output.push_str(&input[i..next_index]);
            stack.push(limited);
            if limited {
                limited_depth += 1;
            }

            i = next_index;
            continue;
        }

        if i + 1 < bytes.len() && bytes[i] == b'}' && bytes[i + 1] == b';' && !stack.is_empty() {
            if stack.pop().unwrap_or(false) {
                limited_depth = limited_depth.saturating_sub(1);
            }
        }

        let ch = input[i..]
            .chars()
            .next()
            .expect("valid UTF-8 char boundary");
        output.push(ch);
        i += ch.len_utf8();
    }

    output
}

fn find_inline_block_end(input: &str, start: usize) -> Option<usize> {
    let mut i = parse_inline_block_start(input, start)?.1;
    let bytes = input.as_bytes();
    let mut depth = 1usize;

    while i < bytes.len() {
        if parse_inline_block_start(input, i).is_some() {
            depth += 1;
            i = parse_inline_block_start(input, i)?.1;
            continue;
        }

        if i + 1 < bytes.len() && bytes[i] == b'}' && bytes[i + 1] == b';' {
            depth -= 1;
            i += 2;
            if depth == 0 {
                return Some(i);
            }
            continue;
        }

        let ch = input[i..].chars().next()?;
        i += ch.len_utf8();
    }

    None
}

fn parse_inline_block_start(input: &str, start: usize) -> Option<(&str, usize)> {
    let bytes = input.as_bytes();
    if bytes.get(start).copied() != Some(b'&') {
        return None;
    }

    let mut j = start + 1;
    while j < bytes.len() && bytes[j].is_ascii_alphabetic() {
        j += 1;
    }

    if j <= start + 1 {
        return None;
    }

    let name = &input[start + 1..j];

    if bytes.get(j).copied() == Some(b'{') {
        return Some((name, j + 1));
    }

    if bytes.get(j).copied() != Some(b'(') {
        return None;
    }

    let mut k = j + 1;
    while k < bytes.len() && bytes[k] != b')' {
        k += 1;
    }

    if k < bytes.len() && bytes.get(k + 1).copied() == Some(b'{') {
        return Some((name, k + 2));
    }

    None
}

fn is_limited_inline_name(name: &str) -> bool {
    matches!(
        name,
        "color" | "size" | "lang" | "abbr" | "ruby" | "time" | "data" | "bdo" | "spoiler"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn apply_second_pass(
        html: &str,
        max_inline_nesting: Option<usize>,
        allow_hex_colors: bool,
        allow_custom_font_size: bool,
    ) -> String {
        let prepared = prepare(html, max_inline_nesting);
        expand(&prepared, allow_hex_colors, allow_custom_font_size)
    }

    fn apply(html: &str) -> String {
        apply_second_pass(html, Some(5), false, false)
    }

    #[test]
    fn test_protect_inline_plugin_with_content() {
        let input = "&test{content};";
        let output = protect_inline_plugins(input);
        assert!(output.contains("INLINE_PLUGIN:test::"));
        assert!(!output.contains("&test"));
    }

    #[test]
    fn test_protect_inline_plugin_with_args_and_content() {
        let input = "&test(arg1,arg2){content};";
        let output = protect_inline_plugins(input);
        assert!(output.contains("INLINE_PLUGIN:test:arg1,arg2:"));
    }

    #[test]
    fn test_skip_html_entities() {
        let input = "&lt; &gt; &amp;";
        let output = protect_inline_plugins(input);
        assert_eq!(input, output); // Should remain unchanged
    }

    // Direct unit tests for the color/size mapping logic itself live with
    // its shared implementation in `decoration_values.rs`. The tests below
    // exercise this module's own responsibility: dispatching `&color()`/
    // `&size()` calls (and the rest of the standard inline plugins) end to
    // end through `apply`/`apply_second_pass`.

    #[test]
    fn test_inline_color_foreground() {
        let input = "This is &color(red){red text};";
        let output = apply(input);
        assert!(output.contains(r#"<span class="umd-color-red">red text</span>"#));
    }

    #[test]
    fn test_inline_color_background() {
        let input = "&color(,yellow){yellow bg};";
        let output = apply(input);
        assert!(output.contains(r#"<span class="umd-bg-yellow">yellow bg</span>"#));
    }

    #[test]
    fn test_inline_color_both() {
        let input = "&color(cyan,yellow){cyan on yellow};";
        let output = apply(input);
        assert!(
            output.contains(r#"class="umd-color-cyan umd-bg-yellow""#),
            "Expected both colors as classes, got: {}",
            output
        );
    }

    #[test]
    fn test_inline_color_invalid() {
        // white and black are not in the UMD palette, so they should be rejected
        let input = "&color(white,black){white on black};";
        let output = apply(input);
        // Invalid colors should be ignored, text remains as-is
        assert_eq!(
            output, "white on black",
            "Invalid colors should be ignored, got: {}",
            output
        );
    }

    #[test]
    fn test_inline_color_hex() {
        // Test with HEX color
        let input = "&color(#FF5733){Custom hex color};";
        let output = apply_second_pass(input, Some(5), true, false);
        assert!(
            output.contains(r#"style="color: #FF5733""#),
            "Expected HEX color as inline style, got: {}",
            output
        );
    }

    #[test]
    fn test_inline_size_keyword() {
        let input = "&size(lg){larger};";
        let output = apply(input);
        assert!(output.contains("<span class=\"umd-text-size-lg\">larger</span>"));
    }

    #[test]
    fn test_inline_size_custom_value_rejected_by_default() {
        let input = "&size(1.5){larger};";
        let output = apply(input);
        assert_eq!(output, "larger");
    }

    #[test]
    fn test_inline_size_custom_value_allowed_when_enabled() {
        let input = "&size(1.5){larger};";
        let output = apply_second_pass(input, Some(5), false, true);
        assert!(output.contains("<span style=\"font-size: 1.5rem\">larger</span>"));
    }

    #[test]
    fn test_inline_sup() {
        let input = "x&sup(2);";
        let output = apply(input);
        assert_eq!(output, "x<sup>2</sup>;");
    }

    #[test]
    fn test_inline_sub() {
        let input = "H&sub(2);O";
        let output = apply(input);
        assert_eq!(output, "H<sub>2</sub>;O");
    }

    #[test]
    fn test_inline_lang() {
        let input = "&lang(en){Hello};";
        let output = apply(input);
        assert_eq!(output, "<span lang=\"en\">Hello</span>;");
    }

    #[test]
    fn test_inline_abbr() {
        let input = "&abbr(HTML){HyperText Markup Language};";
        let output = apply(input);
        assert_eq!(
            output,
            "<abbr title=\"HyperText Markup Language\">HTML</abbr>;"
        );
    }

    #[test]
    fn test_multiple_inline_decorations() {
        let input = "&color(red){Red}; and &size(lg){Big}; and &sup(superscript);";
        let output = apply(input);
        assert!(output.contains("umd-color-red"));
        assert!(output.contains("umd-text-size-lg"));
        assert!(output.contains("<sup>superscript</sup>"));
    }

    #[test]
    fn test_inline_ruby() {
        let input = "&ruby(Ashita){明日};";
        let output = apply(input);
        assert_eq!(
            output,
            "<ruby>明日<rp>(</rp><rt>Ashita</rt><rp>)</rp></ruby>;"
        );
    }

    #[test]
    fn test_semantic_elements() {
        let input = "&dfn(term); &kbd(Ctrl); &samp(output); &var(x);";
        let output = apply(input);
        assert!(output.contains("<dfn>term</dfn>"));
        assert!(output.contains("<kbd>Ctrl</kbd>"));
        assert!(output.contains("<samp>output</samp>"));
        assert!(output.contains("<var>x</var>"));
    }

    #[test]
    fn test_cite_q_small() {
        let input = "&cite(Book Title); &q(quote); &small(note);";
        let output = apply(input);
        assert!(output.contains("<cite>Book Title</cite>"));
        assert!(output.contains("<q>quote</q>"));
        assert!(output.contains("<small>note</small>"));
    }

    #[test]
    fn test_time_and_data() {
        let input = "&time(2026-01-26){today}; &data(12345){value};";
        let output = apply(input);
        assert!(output.contains("<time datetime=\"2026-01-26\">today</time>"));
        assert!(output.contains("<data value=\"12345\">value</data>"));
    }

    #[test]
    fn test_bidirectional_text() {
        let input = "&bdi(مرحبا); &bdo(rtl){right-to-left};";
        let output = apply(input);
        assert!(output.contains("<bdi>مرحبا</bdi>"));
        assert!(output.contains("<bdo dir=\"rtl\">right-to-left</bdo>"));
    }

    #[test]
    fn test_wbr() {
        let input = "Very&wbr;Long&wbr;Word";
        let output = apply(input);
        assert_eq!(output, "Very<wbr />Long<wbr />Word");
    }

    #[test]
    fn test_br() {
        let input = "Line 1&br;Line 2&br;Line 3";
        let output = apply(input);
        assert_eq!(output, "Line 1<br />Line 2<br />Line 3");
    }

    #[test]
    fn test_inline_nesting_depth_detection() {
        let input = "&color(blue){&abbr(text){content};};";
        assert_eq!(inline_decoration_nesting_depth(input), 2);
    }

    #[test]
    fn test_inline_nesting_depth_ignores_plugin_names() {
        let input = "&color(blue){&plugin(arg){content};};";
        assert_eq!(inline_decoration_nesting_depth(input), 1);
    }

    #[test]
    fn test_inline_nesting_limit_blocks_expansion_when_exceeded() {
        let input = "&color(blue){&abbr(text){content};};";
        let output = apply_second_pass(input, Some(1), false, false);
        assert!(output.contains(r#"<span class="umd-color-blue">"#));
        assert!(output.contains(
            r#"<span class="umd-error-deep-recursive">&amp;abbr(text)&#123;content&#125;;</span>"#
        ));
        assert!(!output.contains("<abbr"));
    }

    #[test]
    fn test_inline_nesting_limit_disables_only_exceeded_marker() {
        let input = "&color(red){ok}; and &color(blue){&abbr(text){content};};";
        let output = apply_second_pass(input, Some(1), false, false);
        assert!(output.contains(r#"<span class="umd-color-red">ok</span>"#));
        assert!(output.contains(
            r#"<span class="umd-error-deep-recursive">&amp;abbr(text)&#123;content&#125;;</span>"#
        ));
        assert!(!output.contains("<abbr title="));
    }

    #[test]
    fn test_color_bootstrap_class() {
        let input = "&color(blue){Blue text};";
        let output = apply(input);
        assert!(output.contains("class=\"umd-color-blue\""));
    }

    #[test]
    fn test_color_custom_value() {
        let input = "&color(#FF0000){Red text};";
        let output = apply_second_pass(input, Some(5), true, false);
        assert!(output.contains("style=\"color: #FF0000\""));
    }

    #[test]
    fn test_spoiler_umd_function_parentheses() {
        let input = "This is &spoiler(hidden text); in a sentence.";
        let output = apply(input);
        assert!(output.contains(r#"<span class="umd-spoiler" role="button" tabindex="0" aria-expanded="false">hidden text</span>"#));
    }

    #[test]
    fn test_spoiler_umd_function_braces() {
        let input = "This is &spoiler{hidden text}; in a sentence.";
        let output = apply(input);
        assert!(output.contains(r#"<span class="umd-spoiler" role="button" tabindex="0" aria-expanded="false">hidden text</span>"#));
    }

    #[test]
    fn test_size_keyword_class() {
        let input = "&size(sm){Medium text};";
        let output = apply(input);
        assert!(output.contains("class=\"umd-text-size-sm\""));
    }

    #[test]
    fn test_size_custom_value_allowed_when_enabled() {
        let input = "&size(3rem){Custom size};";
        let output = apply_second_pass(input, Some(5), false, true);
        assert!(output.contains("style=\"font-size: 3rem\""));
    }

    #[test]
    fn test_multiple_spoilers_notation_and_plugin_combined() {
        // Exercises the real pipeline order: notation (||..||) between the
        // two plugin-sweep halves — see `extensions::apply_extensions_with_headers`.
        let input = "||spoiler1|| and ||spoiler2|| and &spoiler{spoiler3};";
        let prepared = prepare(input, Some(5));
        let with_notation = crate::extensions::inline::notation::apply(&prepared);
        let output = expand(&with_notation, false, false);
        let spoiler_count = output.matches(r#"<span class="umd-spoiler""#).count();
        assert_eq!(spoiler_count, 3);
    }
}
