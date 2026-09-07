//! Block decoration syntax for LukiWiki with `umd-*` reference-CSS class support
//!
//! Provides line-prefix decorations with compound syntax support:
//! - COLOR(fg,bg): `umd-color-*`/`umd-bg-*` classes or inline color — value
//!   table and mapping shared with `&color()` via `decoration_values.rs`
//! - SIZE(value): inline `font-size` (rem) — value table and mapping shared
//!   with `&size()` via `decoration_values.rs`
//! - TRUNCATE: `umd-truncate` class
//! - JUSTIFY/START/CENTER/END and V-START/V-CENTER/V-END/BASELINE: alignment
//!   — the mapping/regexes for these live in `alignment.rs` (see that
//!   module's docs); this module only composes them into the full
//!   COLOR/SIZE/TRUNCATE/alignment prefix chain.
//!
//! Multiple prefixes can be combined:
//! - SIZE(lg): COLOR(primary): CENTER: Text
//! - TRUNCATE: END: Text

use once_cell::sync::Lazy;
use regex::Regex;

use super::alignment;
use super::alignment::{ALIGN_EXTRACT, VALIGN_EXTRACT, map_text_align, map_vertical_align};
use super::decoration_values::{map_color_value_with_options, map_font_size_value};

/// Block decoration attributes
#[derive(Default, Debug)]
struct BlockDecoration {
    // Color classes or inline styles
    fg_color: Option<String>,
    bg_color: Option<String>,
    // Font size class or inline style
    font_size: Option<String>,
    // Text alignment class
    text_align: Option<String>,
    // Truncate flag
    truncate: bool,
    // Vertical alignment class (V-START/V-CENTER/V-END/BASELINE); has no
    // visual effect on a plain paragraph, only inside a table cell or other
    // inline-level/table-cell box — kept for prefix-syntax completeness
    vertical_align: Option<String>,
}

impl BlockDecoration {
    /// Convert to HTML class and style attributes
    fn to_html_attrs(&self) -> (Option<String>, Option<String>) {
        let mut classes = Vec::new();
        let mut styles = Vec::new();

        // Text alignment
        if let Some(ref align) = self.text_align {
            classes.push(align.clone());
        }

        // Truncate
        if self.truncate {
            classes.push("umd-truncate".to_string());
        }

        // Vertical alignment
        if let Some(ref valign) = self.vertical_align {
            classes.push(valign.clone());
        }

        // Font size (class or inline)
        if let Some(ref size) = self.font_size {
            if size.starts_with("umd-text-size-") {
                classes.push(size.clone());
            } else {
                styles.push(format!("font-size: {}", size));
            }
        }

        // Foreground color (class or inline)
        if let Some(ref fg) = self.fg_color {
            if fg.starts_with("umd-color-") {
                classes.push(fg.clone());
            } else {
                styles.push(format!("color: {}", fg));
            }
        }

        // Background color (class or inline)
        if let Some(ref bg) = self.bg_color {
            if bg.starts_with("umd-bg-") {
                classes.push(bg.clone());
            } else {
                styles.push(format!("background-color: {}", bg));
            }
        }

        let class_attr = if classes.is_empty() {
            None
        } else {
            Some(format!("class=\"{}\"", classes.join(" ")))
        };

        let style_attr = if styles.is_empty() {
            None
        } else {
            Some(format!("style=\"{}\"", styles.join("; ")))
        };

        (class_attr, style_attr)
    }
}

// Compound prefix pattern: captures all decoration prefixes in one line (reserved for future use)
#[allow(dead_code)]
static COMPOUND_PREFIX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?m)^(?:(?:SIZE\(([^)]+)\)|COLOR\(([^,)]*?)(?:,([^)]*?))?\)|(TRUNCATE)|(V-START|V-CENTER|V-END|BASELINE)|(JUSTIFY|END|CENTER|START)):\s*)+(.+)$"
    )
    .unwrap()
});

// Individual pattern extractors
static SIZE_EXTRACT: Lazy<Regex> = Lazy::new(|| Regex::new(r"SIZE\(([^)]+)\):").unwrap());
static COLOR_EXTRACT: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"COLOR\(([^,)]*?)(?:,([^)]*?))?\):").unwrap());
static TRUNCATE_EXTRACT: Lazy<Regex> = Lazy::new(|| Regex::new(r"(TRUNCATE):").unwrap());

/// Parse all prefixes from a line and extract decoration attributes
fn parse_prefixes_with_options(
    line: &str,
    allow_hex_colors: bool,
    allow_custom_font_size: bool,
) -> (BlockDecoration, String) {
    let mut decoration = BlockDecoration::default();
    let mut remaining = line;

    // Extract SIZE
    if let Some(caps) = SIZE_EXTRACT.captures(remaining) {
        let value = caps.get(1).map_or("", |m| m.as_str());
        decoration.font_size = map_font_size_value(value, allow_custom_font_size).map(|(_, v)| v);
        remaining = &remaining[caps.get(0).unwrap().end()..];
    }

    // Extract COLOR
    if let Some(caps) = COLOR_EXTRACT.captures(remaining) {
        let fg = caps.get(1).map_or("", |m| m.as_str());
        let bg = caps.get(2).map_or("", |m| m.as_str());
        decoration.fg_color =
            map_color_value_with_options(fg, false, allow_hex_colors).map(|(_, v)| v);
        decoration.bg_color =
            map_color_value_with_options(bg, true, allow_hex_colors).map(|(_, v)| v);
        remaining = &remaining[caps.get(0).unwrap().end()..];
    }

    // Extract TRUNCATE
    if TRUNCATE_EXTRACT.is_match(remaining) {
        decoration.truncate = true;
        remaining = TRUNCATE_EXTRACT.replace(remaining, "").to_string().leak();
    }

    // Extract vertical alignment
    if let Some(caps) = VALIGN_EXTRACT.captures(remaining) {
        let value = caps.get(1).map_or("", |m| m.as_str());
        decoration.vertical_align = Some(map_vertical_align(value));
        remaining = &remaining[caps.get(0).unwrap().end()..];
    }

    // Extract text alignment (must be last as it contains the content)
    if let Some(caps) = ALIGN_EXTRACT.captures(remaining) {
        let value = caps.get(1).map_or("", |m| m.as_str());
        decoration.text_align = Some(map_text_align(value));
        remaining = &remaining[caps.get(0).unwrap().end()..];
    }

    (decoration, remaining.trim().to_string())
}

pub fn apply_block_decorations_with_options(
    html: &str,
    allow_hex_colors: bool,
    allow_custom_font_size: bool,
) -> String {
    let mut result = String::new();

    for line in html.lines() {
        let has_alignment_prefix = alignment::TEXT_ALIGN
            .iter()
            .chain(alignment::VERTICAL_ALIGN.iter())
            .any(|(keyword, _)| {
                line.starts_with(keyword) && line[keyword.len()..].starts_with(':')
            });

        if line.starts_with("SIZE(")
            || line.starts_with("COLOR(")
            || line.starts_with("TRUNCATE:")
            || has_alignment_prefix
        {
            let (decoration, content) =
                parse_prefixes_with_options(line, allow_hex_colors, allow_custom_font_size);
            let (class_attr, style_attr) = decoration.to_html_attrs();

            let mut attrs = Vec::new();
            if let Some(class) = class_attr {
                attrs.push(class);
            }
            if let Some(style) = style_attr {
                attrs.push(style);
            }

            if attrs.is_empty() {
                result.push_str(&format!("<p>{}</p>\n", content));
            } else {
                result.push_str(&format!("<p {}>{}</p>\n", attrs.join(" "), content));
            }
        } else {
            result.push_str(line);
            result.push('\n');
        }
    }

    result.trim_end().to_string()
}

pub fn apply_block_decorations(html: &str) -> String {
    apply_block_decorations_with_options(html, false, false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_bootstrap_class() {
        let input = "COLOR(blue): Blue text";
        let output = apply_block_decorations_with_options(input, false, false);
        assert!(output.contains("class=\"umd-color-blue\""));
        assert!(output.contains("Blue text"));
    }

    #[test]
    fn test_color_custom_value() {
        let input = "COLOR(#FF0000): Custom red";
        let output = apply_block_decorations_with_options(input, true, false);
        assert!(output.contains("style=\"color: #FF0000\""));
    }

    #[test]
    fn test_size_keyword_class() {
        let input = "SIZE(lg): Large text";
        let output = apply_block_decorations(input);
        assert!(output.contains("class=\"umd-text-size-lg\""));
    }

    #[test]
    fn test_size_custom_value_rejected_by_default() {
        let input = "SIZE(1.5): Medium text";
        let output = apply_block_decorations(input);
        assert!(!output.contains("font-size"));
        assert!(output.contains("Medium text"));
    }

    #[test]
    fn test_size_custom_value_allowed_when_enabled() {
        let input = "SIZE(3rem): Custom size";
        let output = apply_block_decorations_with_options(input, false, true);
        assert!(output.contains("style=\"font-size: 3rem\""));
    }

    #[test]
    fn test_truncate() {
        let input = "TRUNCATE: Long text that will be truncated";
        let output = apply_block_decorations(input);
        assert!(output.contains("class=\"umd-truncate\""));
    }

    #[test]
    fn test_compound_decorations() {
        let input = "SIZE(lg): COLOR(blue): CENTER: Styled text";
        let output = apply_block_decorations_with_options(input, false, false);
        assert!(output.contains("umd-text-size-lg"));
        assert!(output.contains("umd-color-blue"));
        assert!(output.contains("umd-center"));
        assert!(output.contains("Styled text"));
    }

    #[test]
    fn test_compound_with_truncate() {
        let input = "TRUNCATE: END: Truncated right text";
        let output = apply_block_decorations(input);
        assert!(output.contains("umd-truncate"));
        assert!(output.contains("umd-end"));
    }
}
