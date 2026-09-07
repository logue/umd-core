//! Plain inline notation that isn't part of the `&function()` plugin syntax:
//! - `%%text%%` → `<s>text</s>` (LukiWiki strikethrough)
//! - `||text||` → `<span class="umd-spoiler">` (Discord-style spoiler)
//! - `__text__` → `<u>text</u>` (Discord-style underline, placeholder-based so
//!   CommonMark's `<strong>` conversion never sees it)
//!
//! Standard inline *plugins* (`&color()`, `&size()`, `&spoiler()`, etc.) are
//! handled by `plugins::inline`, not here — see that module's docs for why
//! the plugin sweep is a separate pass.

use once_cell::sync::Lazy;
use regex::Regex;

/// Regex for LukiWiki strikethrough: %%text%% → <s>text</s>
static LUKIWIKI_STRIKETHROUGH: Lazy<Regex> = Lazy::new(|| Regex::new(r"%%([^%]+)%%").unwrap());

/// Regex for Discord-style spoiler: || text || → <span class="umd-spoiler">text</span>
static DISCORD_SPOILER: Lazy<Regex> = Lazy::new(|| Regex::new(r"\|\|([^|]+)\|\|").unwrap());

/// Discord-style underline pattern: __text__
static DISCORD_UNDERLINE: Lazy<Regex> = Lazy::new(|| Regex::new(r"__([^_]+)__").unwrap());

/// Apply plain inline notation (strikethrough, Discord-style spoiler) to HTML.
pub fn apply(html: &str) -> String {
    let mut result = html.to_string();

    // Apply %%text%% → <s>text</s> (LukiWiki strikethrough)
    result = LUKIWIKI_STRIKETHROUGH
        .replace_all(&result, "<s>$1</s>")
        .to_string();

    // Apply || text || → <span class="umd-spoiler">text</span> (Discord spoiler)
    result = DISCORD_SPOILER
        .replace_all(
            &result,
            r#"<span class="umd-spoiler" role="button" tabindex="0" aria-expanded="false">$1</span>"#,
        )
        .to_string();

    result
}

/// Convert Discord-style underline (__text__) to placeholder before Markdown parsing
///
/// This prevents CommonMark from converting __text__ to <strong>
pub fn preprocess_discord_underline(input: &str) -> String {
    DISCORD_UNDERLINE
        .replace_all(input, "{{UNDERLINE:$1:UNDERLINE}}")
        .to_string()
}

/// Restore Discord-style underline placeholders to <u> tags
///
/// This should be called after Markdown parsing
pub fn postprocess_discord_underline(html: &str) -> String {
    html.replace("{{UNDERLINE:", "<u>")
        .replace(":UNDERLINE}}", "</u>")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lukiwiki_strikethrough() {
        let input = "This is %%strikethrough%% text.";
        let output = apply(input);
        assert_eq!(output, "This is <s>strikethrough</s> text.");
    }

    #[test]
    fn test_lukiwiki_strikethrough_multiple() {
        let input = "%%first%% and %%second%%";
        let output = apply(input);
        assert_eq!(output, "<s>first</s> and <s>second</s>");
    }

    #[test]
    fn test_spoiler_discord_syntax() {
        let input = "This is ||hidden text|| in a sentence.";
        let output = apply(input);
        assert!(output.contains(r#"<span class="umd-spoiler" role="button" tabindex="0" aria-expanded="false">hidden text</span>"#));
    }

    #[test]
    fn test_preprocess_discord_underline() {
        let input = "This is __underlined__ text.";
        let output = preprocess_discord_underline(input);
        assert!(output.contains("{{UNDERLINE:underlined:UNDERLINE}}"));
        assert!(!output.contains("__underlined__"));
    }

    #[test]
    fn test_postprocess_discord_underline() {
        let input = "<p>This is {{UNDERLINE:underlined:UNDERLINE}} text.</p>";
        let output = postprocess_discord_underline(input);
        assert_eq!(output, "<p>This is <u>underlined</u> text.</p>");
    }

    #[test]
    fn test_discord_underline_roundtrip() {
        let input = "Text with __underline__ here.";
        let preprocessed = preprocess_discord_underline(input);
        let html = format!(
            "<p>{}</p>",
            preprocessed.replace("__underline__", "{{UNDERLINE:underline:UNDERLINE}}")
        );
        let output = postprocess_discord_underline(&html);
        assert!(output.contains("<u>underline</u>"));
    }
}
