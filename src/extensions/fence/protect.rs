//! Protect fenced code blocks and inline code from other extension passes,
//! then restore them (running fence-specific enhancements on the way back).

use super::code_block;

/// Protect code blocks and inline code from transformation
///
/// Returns the HTML with code sections replaced by placeholders,
/// and a vector of the original code sections.
pub(crate) fn protect_code_sections(html: &str) -> (String, Vec<String>) {
    use regex::Regex;

    let mut placeholders = Vec::new();
    let mut result = html.to_string();

    // Protect <pre><code>...</code></pre> blocks
    let code_block_re = Regex::new(r"<pre><code[^>]*>[\s\S]*?</code></pre>").unwrap();
    result = code_block_re
        .replace_all(&result, |caps: &regex::Captures| {
            let index = placeholders.len();
            placeholders.push(caps[0].to_string());
            format!("<!--CODE_BLOCK_{}-->", index)
        })
        .to_string();

    // Protect <code>...</code> inline
    let inline_code_re = Regex::new(r"<code[^>]*>[^<]*</code>").unwrap();
    result = inline_code_re
        .replace_all(&result, |caps: &regex::Captures| {
            let index = placeholders.len();
            placeholders.push(caps[0].to_string());
            format!("<!--INLINE_CODE_{}-->", index)
        })
        .to_string();

    (result, placeholders)
}

/// Restore protected code sections
pub(crate) fn restore_code_sections(
    html: &str,
    placeholders: &[String],
    color_swatch_icon_html: &str,
) -> String {
    use regex::Regex;

    let mut result = html.to_string();

    // Restore code blocks
    let placeholder_re = Regex::new(r"<!--(CODE_BLOCK|INLINE_CODE)_(\d+)-->").unwrap();
    result = placeholder_re
        .replace_all(&result, |caps: &regex::Captures| {
            let section_type = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            let index: usize = caps[2].parse().unwrap();
            let original = placeholders.get(index).map(|s| s.as_str()).unwrap_or("");

            if section_type == "INLINE_CODE" {
                enhance_inline_code_color_sample(original, color_swatch_icon_html)
            } else {
                original.to_string()
            }
        })
        .to_string();

    // Apply code block enhancements (syntax highlighting, Mermaid, filenames)
    result = code_block::process_code_blocks(&result);

    result
}

fn enhance_inline_code_color_sample(
    inline_code_html: &str,
    color_swatch_icon_html: &str,
) -> String {
    use once_cell::sync::Lazy;
    use regex::Regex;

    static INLINE_CODE_TAG_RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r#"^<code(?P<attrs>[^>]*)>(?P<content>[^<]*)</code>$"#)
            .expect("valid inline code tag regex")
    });
    static HEX_COLOR_RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"(?i)^#(?:[0-9a-f]{3}|[0-9a-f]{4}|[0-9a-f]{6}|[0-9a-f]{8})$")
            .expect("valid hex color regex")
    });
    static RGB_COLOR_RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(
            r"(?i)^rgb\(\s*(?:25[0-5]|2[0-4]\d|1?\d?\d)\s*,\s*(?:25[0-5]|2[0-4]\d|1?\d?\d)\s*,\s*(?:25[0-5]|2[0-4]\d|1?\d?\d)\s*\)$",
        )
        .expect("valid rgb color regex")
    });
    static RGBA_COLOR_RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(
            r"(?i)^rgba\(\s*(?:25[0-5]|2[0-4]\d|1?\d?\d)\s*,\s*(?:25[0-5]|2[0-4]\d|1?\d?\d)\s*,\s*(?:25[0-5]|2[0-4]\d|1?\d?\d)\s*,\s*(?:0|1|0?\.\d+|1(?:\.0+)?)\s*\)$",
        )
        .expect("valid rgba color regex")
    });
    static HSL_COLOR_RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(
            r"(?i)^hsl\(\s*(?:360|3[0-5]\d|[12]?\d?\d)\s*,\s*(?:100|[1-9]?\d)%\s*,\s*(?:100|[1-9]?\d)%\s*\)$",
        )
        .expect("valid hsl color regex")
    });
    static HSLA_COLOR_RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(
            r"(?i)^hsla\(\s*(?:360|3[0-5]\d|[12]?\d?\d)\s*,\s*(?:100|[1-9]?\d)%\s*,\s*(?:100|[1-9]?\d)%\s*,\s*(?:0|1|0?\.\d+|1(?:\.0+)?)\s*\)$",
        )
        .expect("valid hsla color regex")
    });

    let Some(caps) = INLINE_CODE_TAG_RE.captures(inline_code_html) else {
        return inline_code_html.to_string();
    };

    let attrs = caps.name("attrs").map(|m| m.as_str()).unwrap_or("");
    let content = caps.name("content").map(|m| m.as_str()).unwrap_or("");
    let color_text = content.trim();

    let is_color_code = HEX_COLOR_RE.is_match(color_text)
        || RGB_COLOR_RE.is_match(color_text)
        || RGBA_COLOR_RE.is_match(color_text)
        || HSL_COLOR_RE.is_match(color_text)
        || HSLA_COLOR_RE.is_match(color_text);

    if !is_color_code {
        return inline_code_html.to_string();
    }

    format!(
        r#"<code{}><span class="inline-code-color" style="background-color: {};">{}</span>{}</code>"#,
        attrs, color_text, color_swatch_icon_html, content
    )
}
