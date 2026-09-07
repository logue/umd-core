//! Shared value tables and mapping logic for `COLOR()`/`&color()` and
//! `SIZE()`/`&size()`.
//!
//! The block-decoration prefix syntax (`block_decoration.rs`) and the
//! inline plugin syntax (`plugins::inline`) both accept the same color
//! palette and the same font-size keywords, so there is exactly one
//! definition of "what's a valid value" and one implementation of how it
//! maps to a class or an inline style. Keeping this single-sourced matters
//! beyond just avoiding copy-paste: `plugins::inline`'s second-pass sweep
//! must produce byte-identical output to its own first-pass dispatch for
//! the same input, and a second, independently-maintained copy of this
//! mapping would risk exactly that kind of drift.

/// UMD's named color palette (`umd-color-*` / `umd-bg-*`).
const COLOR_NAMES: &[&str] = &[
    "blue", "indigo", "violet", "purple", "pink", "red", "orange", "amber", "yellow", "lime",
    "green", "teal", "cyan", "brown", "gray", "pewter",
];

/// `SIZE()`/`&size()` keyword sizes (`umd-text-size-*`). Values outside this
/// set are only accepted as an arbitrary inline `font-size` when
/// `allow_custom_font_size` is enabled — there's no discrete class for
/// arbitrary numeric values, and allowing unbounded custom sizes is opt-in
/// to prevent abuse in untrusted content.
const FONT_SIZE_KEYWORDS: &[&str] = &["xs", "sm", "lg", "xl"];

/// Map a color value to a `umd-color-*`/`umd-bg-*` class, or (only when
/// `allow_hex_colors` is enabled) a `#rgb`/`#rrggbb` inline style value.
/// Returns `Some((is_class, value))`, or `None` if the value is rejected.
pub(crate) fn map_color_value_with_options(
    value: &str,
    is_background: bool,
    allow_hex_colors: bool,
) -> Option<(bool, String)> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed == "inherit" {
        return None;
    }

    if COLOR_NAMES.contains(&trimmed) {
        let prefix = if is_background { "umd-bg" } else { "umd-color" };
        return Some((true, format!("{}-{}", prefix, trimmed)));
    }

    if allow_hex_colors
        && trimmed.starts_with('#')
        && (trimmed.len() == 4 || trimmed.len() == 7)
        && trimmed[1..].chars().all(|c| c.is_ascii_hexdigit())
    {
        return Some((false, trimmed.to_string()));
    }

    // Invalid color - reject
    None
}

/// Map a `SIZE()`/`&size()` value to a `umd-text-size-*` class, or (only
/// when `allow_custom_font_size` is enabled) an arbitrary inline
/// `font-size`. Returns `Some((is_class, value))`, or `None` if rejected.
pub(crate) fn map_font_size_value(
    value: &str,
    allow_custom_font_size: bool,
) -> Option<(bool, String)> {
    let trimmed = value.trim();

    if FONT_SIZE_KEYWORDS.contains(&trimmed) {
        return Some((true, format!("umd-text-size-{}", trimmed)));
    }

    if !allow_custom_font_size {
        return None;
    }

    if trimmed.contains("rem") || trimmed.contains("em") || trimmed.contains("px") {
        return Some((false, trimmed.to_string()));
    }

    Some((false, format!("{}rem", trimmed)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_color_blue() {
        let result = map_color_value_with_options("blue", false, false);
        assert!(
            result.is_some(),
            "blue should be recognized as a valid color"
        );
        let (is_class, class_or_style) = result.unwrap();
        assert!(is_class, "blue should be recognized as a umd-color-* class");
        assert_eq!(
            class_or_style, "umd-color-blue",
            "Expected umd-color-blue, got {}",
            class_or_style
        );
    }

    #[test]
    fn test_map_color_background_prefix() {
        let result = map_color_value_with_options("yellow", true, false);
        assert_eq!(result, Some((true, "umd-bg-yellow".to_string())));
    }

    #[test]
    fn test_map_color_hex() {
        let result = map_color_value_with_options("#FF5733", false, true);
        assert!(
            result.is_some(),
            "#FF5733 should be recognized as a valid HEX color"
        );
        let (is_class, value) = result.unwrap();
        assert!(!is_class, "HEX color should not be a umd-color-* class");
        assert_eq!(value, "#FF5733", "Expected #FF5733, got {}", value);
    }

    #[test]
    fn test_map_color_hex_rejected_by_default() {
        let result = map_color_value_with_options("#FF5733", false, false);
        assert!(result.is_none());
    }

    #[test]
    fn test_map_color_invalid_html_name() {
        // HTML color names like "white" or "black" are not in the UMD
        // palette and should be rejected
        let result = map_color_value_with_options("white", false, false);
        assert!(
            result.is_none(),
            "HTML color name 'white' should be rejected"
        );

        let result = map_color_value_with_options("black", false, false);
        assert!(
            result.is_none(),
            "HTML color name 'black' should be rejected"
        );
    }

    #[test]
    fn test_map_color_empty_and_inherit_rejected() {
        assert!(map_color_value_with_options("", false, false).is_none());
        assert!(map_color_value_with_options("inherit", false, false).is_none());
    }

    #[test]
    fn test_map_font_size_keyword() {
        assert_eq!(
            map_font_size_value("lg", false),
            Some((true, "umd-text-size-lg".to_string()))
        );
    }

    #[test]
    fn test_map_font_size_custom_rejected_by_default() {
        assert_eq!(map_font_size_value("1.5", false), None);
    }

    #[test]
    fn test_map_font_size_custom_allowed_when_enabled() {
        assert_eq!(
            map_font_size_value("1.5", true),
            Some((false, "1.5rem".to_string()))
        );
        assert_eq!(
            map_font_size_value("14px", true),
            Some((false, "14px".to_string()))
        );
    }
}
