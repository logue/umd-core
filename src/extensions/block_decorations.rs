//! Block decoration syntax for LukiWiki with `umd-*` reference-CSS class support
//!
//! Provides line-prefix decorations with compound syntax support:
//! - COLOR(fg,bg): `umd-color-*`/`umd-bg-*` classes or inline color
//! - SIZE(value): inline `font-size` (rem)
//! - TRUNCATE: `umd-truncate` class
//! - JUSTIFY/START/CENTER/END: inline-direction `umd-*` text alignment classes
//! - V-START/V-CENTER/V-END/BASELINE: block-direction `umd-v-*` alignment classes
//!
//! `START`/`END` and `V-START`/`V-CENTER`/`V-END` are logical-direction names
//! (matching `scss/utilities/text.scss`), not physical `LEFT`/`RIGHT`/`TOP`/
//! `BOTTOM` — under `dir="rtl"` or vertical writing modes, "start" and "end"
//! (or "block-start"/"block-end") flip automatically, whereas a hardcoded
//! `LEFT`/`TOP` would not. See PLAN.md's "CSS 論理プロパティ方針".
//!
//! The table/plugin block-placement wrapper (`apply_block_placement`, and
//! table cell alignment in `conflict_resolver.rs`/`table/umd/decorations.rs`)
//! is a separate, still-physical-named (`LEFT`/`RIGHT`/`TOP`/`BOTTOM`) system
//! — out of scope for this pass, tracked in PLAN.md.
//!
//! Multiple prefixes can be combined:
//! - SIZE(lg): COLOR(primary): CENTER: Text
//! - TRUNCATE: END: Text

use once_cell::sync::Lazy;
use regex::Regex;

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
static VALIGN_EXTRACT: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(V-START|V-CENTER|V-END|BASELINE):").unwrap());
static ALIGN_EXTRACT: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(JUSTIFY|END|CENTER|START):").unwrap());

// Block placement pattern for tables and plugins (must start on new line)
static BLOCK_PLACEMENT: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?m)^(START|CENTER|END|JUSTIFY):\n((?:\|[^\n]*\|(?:\n|$))+|@\w+(?:\([^)]*\))?\{[^}]*\})",
    )
    .unwrap()
});

/// Map a `SIZE()` value to a `umd-text-size-*` class, or (only when
/// `allow_custom_font_size` is enabled) an arbitrary inline `font-size`.
///
/// By default only the keyword sizes `xs`/`sm`/`lg`/`xl` are accepted —
/// there's no discrete class for arbitrary numeric values, and allowing
/// unbounded custom sizes is opt-in to prevent abuse in untrusted content.
fn map_font_size(value: &str, allow_custom_font_size: bool) -> Option<String> {
    let trimmed = value.trim();

    if matches!(trimmed, "xs" | "sm" | "lg" | "xl") {
        return Some(format!("umd-text-size-{}", trimmed));
    }

    if !allow_custom_font_size {
        return None;
    }

    if trimmed.contains("rem") || trimmed.contains("em") || trimmed.contains("px") {
        Some(trimmed.to_string())
    } else {
        Some(format!("{}rem", trimmed))
    }
}

/// Map color value to a `umd-color-*`/`umd-bg-*` class or inline style
fn map_color(value: &str, is_background: bool) -> Option<String> {
    map_color_with_options(value, is_background, false)
}

fn map_color_with_options(
    value: &str,
    is_background: bool,
    allow_hex_colors: bool,
) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed == "inherit" {
        return None;
    }

    let colors = [
        "blue", "indigo", "violet", "purple", "pink", "red", "orange", "amber", "yellow", "lime",
        "green", "teal", "cyan", "brown", "gray", "pewter",
    ];
    for color in colors {
        if trimmed == color {
            let prefix = if is_background { "umd-bg" } else { "umd-color" };
            return Some(format!("{}-{}", prefix, trimmed));
        }
    }

    if allow_hex_colors
        && trimmed.starts_with('#')
        && (trimmed.len() == 4 || trimmed.len() == 7)
        && trimmed[1..].chars().all(|c| c.is_ascii_hexdigit())
    {
        return Some(trimmed.to_string());
    }

    None
}

/// Map inline-direction alignment (`START`/`CENTER`/`END`/`JUSTIFY`) to a
/// `umd-*` text-align class
fn map_text_align(value: &str) -> String {
    match value.to_uppercase().as_str() {
        "END" => "umd-end".to_string(),
        "CENTER" => "umd-center".to_string(),
        "START" => "umd-start".to_string(),
        "JUSTIFY" => "umd-justify".to_string(),
        _ => "umd-start".to_string(),
    }
}

/// Map block-direction alignment (`V-START`/`V-CENTER`/`V-END`/`BASELINE`)
/// to a `umd-v-*` class. There's no CSS logical equivalent of
/// `vertical-align` — `V-` is UMD's own naming convention.
fn map_vertical_align(value: &str) -> String {
    match value.to_uppercase().as_str() {
        "V-START" => "umd-v-start".to_string(),
        "V-CENTER" => "umd-v-center".to_string(),
        "V-END" => "umd-v-end".to_string(),
        "BASELINE" => "umd-v-baseline".to_string(),
        _ => "umd-v-baseline".to_string(),
    }
}

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
        decoration.font_size = map_font_size(value, allow_custom_font_size);
        remaining = &remaining[caps.get(0).unwrap().end()..];
    }

    // Extract COLOR
    if let Some(caps) = COLOR_EXTRACT.captures(remaining) {
        let fg = caps.get(1).map_or("", |m| m.as_str());
        let bg = caps.get(2).map_or("", |m| m.as_str());
        decoration.fg_color = map_color_with_options(
            fg,
            false,
            allow_hex_colors,
        );
        decoration.bg_color = map_color_with_options(
            bg,
            true,
            allow_hex_colors,
        );
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

fn parse_prefixes(line: &str) -> (BlockDecoration, String) {
    parse_prefixes_with_options(line, false, false)
}

pub fn apply_block_decorations_with_options(
    html: &str,
    allow_hex_colors: bool,
    allow_custom_font_size: bool,
) -> String {
    let mut result = String::new();

    for line in html.lines() {
        if line.starts_with("SIZE(")
            || line.starts_with("COLOR(")
            || line.starts_with("TRUNCATE:")
            || line.starts_with("V-START:")
            || line.starts_with("V-CENTER:")
            || line.starts_with("V-END:")
            || line.starts_with("BASELINE:")
            || line.starts_with("JUSTIFY:")
            || line.starts_with("END:")
            || line.starts_with("CENTER:")
            || line.starts_with("START:")
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

/// Apply block placement prefixes to tables and block plugins
///
/// Handles START:/CENTER:/END:/JUSTIFY: prefixes followed by newline
/// for UMD tables and block plugins (@function). Logical-direction names
/// (matching block_decorations.rs's paragraph alignment and
/// scss/utilities/text.scss), not physical LEFT:/RIGHT:.
///
/// # Arguments
///
/// * `html` - The HTML content to process
///
/// # Returns
///
/// HTML with block placement applied (`umd-*` reference-CSS classes)
pub fn apply_block_placement(html: &str) -> String {
    fn merge_class_attr(tag_html: &str, extra_classes: &str) -> String {
        let class_re = Regex::new(r#"class=\"([^\"]*)\""#).unwrap();

        if let Some(caps) = class_re.captures(tag_html) {
            let existing = caps.get(1).map_or("", |m| m.as_str());
            let mut merged: Vec<String> = if existing.trim().is_empty() {
                Vec::new()
            } else {
                existing.split_whitespace().map(|s| s.to_string()).collect()
            };

            for class_name in extra_classes.split_whitespace() {
                if !merged.iter().any(|c| c == class_name) {
                    merged.push(class_name.to_string());
                }
            }

            class_re
                .replace(tag_html, format!(r#"class=\"{}\""#, merged.join(" ")))
                .to_string()
        } else {
            tag_html.replacen('>', &format!(r#" class=\"{}\">"#, extra_classes), 1)
        }
    }

    fn placement_class_for_block(placement: &str) -> &'static str {
        match placement {
            "START" => "umd-block-auto",
            "CENTER" => "umd-block-center",
            "END" => "umd-block-auto umd-block-end",
            "JUSTIFY" => "umd-block-justify",
            _ => "",
        }
    }

    let media_block_placement = Regex::new(
        r#"(?s)<p>\s*(START|CENTER|END|JUSTIFY):\s*\n\s*(<picture[\s\S]*?</picture>|<video[\s\S]*?</video>|<audio[\s\S]*?</audio>|<a href="[^"]+" download class="download-link[^"]*"[^>]*>[\s\S]*?</a>)\s*</p>"#,
    )
    .unwrap();

    let with_media_placement = media_block_placement
        .replace_all(html, |caps: &regex::Captures| {
            let placement = &caps[1];
            let media = &caps[2];

            let wrapper_class = match placement {
                "START" => "umd-block-start",
                "CENTER" => "umd-inline-center",
                "END" => "umd-block-end",
                "JUSTIFY" => "umd-block-justify",
                _ => "",
            };

            if wrapper_class.is_empty() {
                format!("<figure>\n{}\n</figure>", media)
            } else {
                format!("<figure class=\"{}\">\n{}\n</figure>", wrapper_class, media)
            }
        })
        .to_string();

    let table_and_plugin_placement_in_paragraph = Regex::new(
        r#"(?s)<p>\s*(START|CENTER|END|JUSTIFY):\s*\n\s*(<(?:table|template)\b[^>]*>[\s\S]*?</(?:table|template)>)\s*</p>"#,
    )
    .unwrap();

    let with_table_and_plugin_placement_in_paragraph = table_and_plugin_placement_in_paragraph
        .replace_all(&with_media_placement, |caps: &regex::Captures| {
            let placement = &caps[1];
            let block = &caps[2];
            let placement_class = placement_class_for_block(placement);

            if block.starts_with("<table") {
                return merge_class_attr(block, placement_class);
            }

            if block.starts_with("<template") && block.contains("umd-plugin") {
                return merge_class_attr(block, placement_class);
            }

            block.to_string()
        })
        .to_string();

    let table_and_plugin_placement = Regex::new(
        r#"(?s)<p>\s*(START|CENTER|END|JUSTIFY):\s*</p>\s*(<(?:table|template)\b[^>]*>[\s\S]*?</(?:table|template)>)"#,
    )
    .unwrap();

    let with_table_and_plugin_placement = table_and_plugin_placement
        .replace_all(
            &with_table_and_plugin_placement_in_paragraph,
            |caps: &regex::Captures| {
                let placement = &caps[1];
                let block = &caps[2];
                let placement_class = placement_class_for_block(placement);

                if block.starts_with("<table") {
                    return merge_class_attr(block, placement_class);
                }

                if block.starts_with("<template") && block.contains("umd-plugin") {
                    return merge_class_attr(block, placement_class);
                }

                block.to_string()
            },
        )
        .to_string();

    BLOCK_PLACEMENT
        .replace_all(
            &with_table_and_plugin_placement,
            |caps: &regex::Captures| {
                let placement = &caps[1];
                let content = &caps[2];

                let wrapper_class = placement_class_for_block(placement);

                // Wrap table or plugin in div with appropriate class
                if content.starts_with('|') {
                    // UMD table
                    format!("<div class=\"{}\">\n{}</div>", wrapper_class, content)
                } else if content.starts_with('@') {
                    // Block plugin
                    format!("<div class=\"{}\">\n{}</div>", wrapper_class, content)
                } else {
                    content.to_string()
                }
            },
        )
        .to_string()
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
    fn test_text_align() {
        let input = "CENTER: Centered text";
        let output = apply_block_decorations(input);
        assert!(output.contains("class=\"umd-center\""));
    }

    #[test]
    fn test_text_align_start() {
        let input = "START: Start-aligned text";
        let output = apply_block_decorations(input);
        assert!(output.contains("class=\"umd-start\""));
    }

    #[test]
    fn test_text_align_end() {
        let input = "END: End-aligned text";
        let output = apply_block_decorations(input);
        assert!(output.contains("class=\"umd-end\""));
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
    fn test_vertical_align() {
        let input = "V-START: Top aligned";
        let output = apply_block_decorations(input);
        assert!(output.contains("class=\"umd-v-start\""));
    }

    #[test]
    fn test_vertical_align_center_and_end() {
        assert!(
            apply_block_decorations("V-CENTER: Middle aligned").contains("class=\"umd-v-center\"")
        );
        assert!(apply_block_decorations("V-END: Bottom aligned").contains("class=\"umd-v-end\""));
    }

    #[test]
    fn test_vertical_align_baseline() {
        let input = "BASELINE: Baseline aligned";
        let output = apply_block_decorations(input);
        assert!(output.contains("class=\"umd-v-baseline\""));
    }

    #[test]
    fn test_compound_with_truncate() {
        let input = "TRUNCATE: END: Truncated right text";
        let output = apply_block_decorations(input);
        assert!(output.contains("umd-truncate"));
        assert!(output.contains("umd-end"));
    }

    #[test]
    fn test_block_placement_start() {
        let input = "START:\n|Header|\n|Cell|";
        let output = apply_block_placement(input);
        assert!(output.contains(r#"<div class="umd-block-auto">"#));
        assert!(output.contains("|Header|"));
    }

    #[test]
    fn test_block_placement_center() {
        let input = "CENTER:\n|Header|\n|Cell|";
        let output = apply_block_placement(input);
        assert!(output.contains(r#"<div class="umd-block-center">"#));
    }

    #[test]
    fn test_block_placement_end() {
        let input = "END:\n|Header|\n|Cell|";
        let output = apply_block_placement(input);
        assert!(output.contains(r#"<div class="umd-block-auto umd-block-end">"#));
    }

    #[test]
    fn test_block_placement_justify() {
        let input = "JUSTIFY:\n|Header|\n|Cell|";
        let output = apply_block_placement(input);
        assert!(output.contains(r#"<div class="umd-block-justify">"#));
    }

    #[test]
    fn test_block_placement_plugin() {
        let input = "CENTER:\n@youtube{video_id}";
        let output = apply_block_placement(input);
        assert!(output.contains(r#"<div class="umd-block-center">"#));
        assert!(output.contains("@youtube"));
    }

    #[test]
    fn test_block_placement_end_media() {
        let input = r#"<p>END:
<picture>
  <img src="image.png" alt="alt" title="Title" />
</picture></p>"#;
        let output = apply_block_placement(input);
        assert!(output.contains(r#"<figure class="umd-block-end">"#));
        assert!(output.contains("<picture>"));
        assert!(!output.contains("END:"));
    }
}
