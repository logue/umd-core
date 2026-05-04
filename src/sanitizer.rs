//! HTML sanitization module
//!
//! This module provides HTML sanitization functionality to prevent XSS attacks.
//! It escapes all HTML tags in user input while preserving standard HTML entities.
//! It also blocks dangerous URL schemes.

use std::borrow::Cow;

/// Sanitizes a URL by blocking dangerous schemes
///
/// # Arguments
///
/// * `url` - The URL to sanitize
///
/// # Returns
///
/// A sanitized URL or `#blocked-url` if the scheme is blocked
///
/// # Blocked Schemes
///
/// - `javascript:` - JavaScript execution XSS
/// - `data:` - Base64 encoded script injection XSS
/// - `vbscript:` - VBScript execution XSS (IE legacy)
/// - `file:` - Local file system access (information leakage)
///
/// Note: `file:` scheme is blocked by default for security reasons, but may be
/// useful in specific use cases such as:
/// - Standalone software offline help systems
/// - Local document management applications
/// - Electron/Tauri apps with local resource access
///
/// Future enhancement: Consider adding a configuration option to allow `file:`
/// scheme when explicitly enabled by the application developer (see planned-features.md).
///
/// # Behavior
///
/// When a dangerous scheme is detected:
/// - In explicit autolinks (`<url>`): the URL is rendered as plain text (not linked)
/// - In inline links (`[text](url)`): the link is replaced with `#blocked-url` for safety
///
/// Allowed schemes include:
/// - Standard protocols: `http:`, `https:`, `mailto:`, `tel:`, `ftp:`
/// - Custom app schemes: `spotify:`, `discord:`, `vscode:`, `steam:`, etc.
/// - Relative paths: `/path`, `./path`, `#anchor`
///
/// # Examples
///
/// ```
/// use umd::sanitizer::sanitize_url;
///
/// assert_eq!(sanitize_url("https://example.com"), "https://example.com");
/// assert_eq!(sanitize_url("javascript:alert(1)"), "#blocked-url");
/// assert_eq!(sanitize_url("data:text/html,<script>alert(1)</script>"), "#blocked-url");
/// assert_eq!(sanitize_url("spotify:track:123"), "spotify:track:123"); // Custom app schemes allowed
/// ```
pub fn sanitize_url(url: &str) -> Cow<'_, str> {
    let normalized = remove_disallowed_blank_chars(url);
    let url_lower = normalized.trim().to_lowercase();

    // Check for dangerous schemes (case-insensitive)
    // TODO: Consider adding ParserOptions.allow_file_scheme configuration
    // to conditionally allow file:// in trusted environments (see planned-features.md)
    if url_lower.starts_with("javascript:")
        || url_lower.starts_with("data:")
        || url_lower.starts_with("vbscript:")
        || url_lower.starts_with("file:")
    {
        return Cow::Borrowed("#blocked-url");
    }

    normalized
}

/// Sanitizes input text by escaping HTML tags while preserving HTML entities
/// and removing disallowed invisible blank-like characters.
///
/// # Arguments
///
/// * `input` - The raw input text to sanitize
///
/// # Returns
///
/// A sanitized string with HTML tags escaped but entities preserved
/// (while removing disallowed invisible blank-like characters).
///
/// # Examples
///
/// ```
/// use umd::sanitizer::sanitize;
///
/// let input = "<script>alert('xss')</script>";
/// let output = sanitize(input);
/// assert_eq!(output, "&lt;script&gt;alert('xss')&lt;/script&gt;");
///
/// // HTML entities are preserved
/// let input_with_entity = "Hello&nbsp;World &lt;tag&gt;";
/// let output = sanitize(input_with_entity);
/// assert_eq!(output, "Hello&nbsp;World &lt;tag&gt;");
/// ```
pub fn sanitize(input: &str) -> Cow<'_, str> {
    let normalized = remove_disallowed_blank_chars(input);
    let source = normalized.as_ref();

    // Check if input contains any characters that need escaping
    if !source.contains(&['<', '>', '&'][..]) {
        return normalized;
    }

    let mut result = String::with_capacity(source.len() + 32);
    let mut chars = source.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            '&' => {
                // Check if this is an HTML entity
                if is_html_entity(&mut chars.clone()) {
                    // Preserve the entity
                    result.push(ch);
                } else {
                    // Escape the ampersand
                    result.push_str("&amp;");
                }
            }
            _ => result.push(ch),
        }
    }

    Cow::Owned(result)
}

fn remove_disallowed_blank_chars(input: &str) -> Cow<'_, str> {
    if !input.chars().any(is_disallowed_blank_char) {
        return Cow::Borrowed(input);
    }

    let filtered: String = input
        .chars()
        .filter(|&ch| !is_disallowed_blank_char(ch))
        .collect();

    Cow::Owned(filtered)
}

fn is_disallowed_blank_char(ch: char) -> bool {
    matches!(
        ch,
        '\u{200B}' // Zero Width Space
            | '\u{200C}' // Zero Width Non-Joiner
            | '\u{200D}' // Zero Width Joiner
            | '\u{FEFF}' // Zero Width No-Break Space / BOM
            | '\u{3164}' // Hangul Filler
    )
}

/// Checks if the character sequence starting with '&' is a valid HTML entity
///
/// Valid entities are:
/// - Named entities: &name; (e.g., &nbsp;, &lt;, &gt;, &amp;, &quot;, &apos;)
/// - Decimal entities: &#123;
/// - Hexadecimal entities: &#x7B;
fn is_html_entity(chars: &mut std::iter::Peekable<std::str::Chars>) -> bool {
    let mut entity = String::new();
    let mut temp_chars = chars.clone();

    // Skip the '&' itself
    while let Some(&ch) = temp_chars.peek() {
        if ch == ';' {
            // Found the end of entity
            return is_valid_entity(&entity);
        }
        if entity.len() > 10 {
            // Entity too long, not valid
            return false;
        }
        if !ch.is_alphanumeric() && ch != '#' && ch != 'x' && ch != 'X' {
            // Invalid character in entity
            return false;
        }
        entity.push(ch);
        temp_chars.next();
    }

    false
}

/// Validates if the entity name (without & and ;) is a valid HTML entity
fn is_valid_entity(entity: &str) -> bool {
    if entity.is_empty() {
        return false;
    }

    // Numeric entities
    if entity.starts_with('#') {
        if entity.len() < 2 {
            return false;
        }
        if entity[1..].starts_with('x') || entity[1..].starts_with('X') {
            // Hexadecimal: &#xHH;
            if entity.len() < 3 {
                return false;
            }
            return entity[2..].chars().all(|c| c.is_ascii_hexdigit());
        } else {
            // Decimal: &#123;
            return entity[1..].chars().all(|c| c.is_ascii_digit());
        }
    }

    // Named entities - common ones
    // Full list: https://html.spec.whatwg.org/multipage/named-characters.html
    matches!(
        entity,
        "nbsp"
            | "lt"
            | "gt"
            | "amp"
            | "quot"
            | "apos"
            | "copy"
            | "reg"
            | "trade"
            | "ndash"
            | "mdash"
            | "lsquo"
            | "rsquo"
            | "ldquo"
            | "rdquo"
            | "hellip"
            | "prime"
            | "Prime"
            | "euro"
            | "yen"
            | "pound"
            | "cent"
            | "times"
            | "divide"
            | "plusmn"
            | "minus"
            | "alpha"
            | "beta"
            | "gamma"
            | "delta"
            | "epsilon"
            | "Alpha"
            | "Beta"
            | "Gamma"
            | "Delta"
            | "Epsilon" // Add more as needed
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_html() {
        let input = "Hello World";
        assert_eq!(sanitize(input), "Hello World");
    }

    #[test]
    fn test_remove_disallowed_blank_like_chars() {
        let input = "A\u{200B}B\u{200C}C\u{200D}D\u{FEFF}E\u{3164}F";
        assert_eq!(sanitize(input), "ABCDEF");
    }

    #[test]
    fn test_preserve_allowed_spaces_only() {
        let input = "A B　C";
        assert_eq!(sanitize(input), "A B　C");
    }

    #[test]
    fn test_escape_tags() {
        let input = "<script>alert('xss')</script>";
        assert_eq!(sanitize(input), "&lt;script&gt;alert('xss')&lt;/script&gt;");
    }

    #[test]
    fn test_preserve_entities() {
        let input = "Hello&nbsp;World";
        assert_eq!(sanitize(input), "Hello&nbsp;World");
    }

    #[test]
    fn test_escape_ampersand() {
        let input = "A & B";
        assert_eq!(sanitize(input), "A &amp; B");
    }

    #[test]
    fn test_mixed_content() {
        let input = "<div>Hello&nbsp;World &amp; stuff</div>";
        assert_eq!(
            sanitize(input),
            "&lt;div&gt;Hello&nbsp;World &amp; stuff&lt;/div&gt;"
        );
    }

    #[test]
    fn test_numeric_entities() {
        let input = "&#123; &#x7B;";
        assert_eq!(sanitize(input), "&#123; &#x7B;");
    }

    #[test]
    fn test_invalid_entity() {
        let input = "&invalid;";
        assert_eq!(sanitize(input), "&amp;invalid;");
    }

    #[test]
    fn test_xss_attempts() {
        let test_cases = vec![
            (
                "<img src=x onerror=alert(1)>",
                "&lt;img src=x onerror=alert(1)&gt;",
            ),
            ("<svg/onload=alert(1)>", "&lt;svg/onload=alert(1)&gt;"),
            (
                "<iframe src=javascript:alert(1)>",
                "&lt;iframe src=javascript:alert(1)&gt;",
            ),
        ];

        for (input, expected) in test_cases {
            assert_eq!(sanitize(input), expected);
        }
    }

    #[test]
    fn test_entity_validation() {
        assert!(is_valid_entity("nbsp"));
        assert!(is_valid_entity("lt"));
        assert!(is_valid_entity("gt"));
        assert!(is_valid_entity("#123"));
        assert!(is_valid_entity("#x7B"));
        assert!(!is_valid_entity("invalid"));
        assert!(!is_valid_entity(""));
    }

    #[test]
    fn test_sanitize_url_safe_schemes() {
        assert_eq!(sanitize_url("https://example.com"), "https://example.com");
        assert_eq!(sanitize_url("http://example.com"), "http://example.com");
        assert_eq!(
            sanitize_url("mailto:user@example.com"),
            "mailto:user@example.com"
        );
        assert_eq!(sanitize_url("ftp://example.com"), "ftp://example.com");
        assert_eq!(sanitize_url("/relative/path"), "/relative/path");
        assert_eq!(sanitize_url("./relative"), "./relative");
        assert_eq!(sanitize_url("#anchor"), "#anchor");
    }

    #[test]
    fn test_sanitize_url_custom_app_schemes() {
        assert_eq!(sanitize_url("spotify:track:123"), "spotify:track:123");
        assert_eq!(sanitize_url("steam://open/game"), "steam://open/game");
        assert_eq!(sanitize_url("discord://invite/123"), "discord://invite/123");
        assert_eq!(
            sanitize_url("slack://channel?id=123"),
            "slack://channel?id=123"
        );
        assert_eq!(sanitize_url("zoom:meeting:123"), "zoom:meeting:123");
        assert_eq!(sanitize_url("vscode://file/path"), "vscode://file/path");
    }

    #[test]
    fn test_sanitize_url_blocked_schemes() {
        assert_eq!(sanitize_url("javascript:alert(1)"), "#blocked-url");
        assert_eq!(sanitize_url("JavaScript:alert(1)"), "#blocked-url");
        assert_eq!(sanitize_url("JAVASCRIPT:alert(1)"), "#blocked-url");
        assert_eq!(
            sanitize_url("data:text/html,<script>alert(1)</script>"),
            "#blocked-url"
        );
        assert_eq!(sanitize_url("Data:text/html,test"), "#blocked-url");
        assert_eq!(sanitize_url("vbscript:msgbox(1)"), "#blocked-url");
        assert_eq!(sanitize_url("VBScript:msgbox(1)"), "#blocked-url");
        assert_eq!(sanitize_url("file:///etc/passwd"), "#blocked-url");
        assert_eq!(sanitize_url("FILE:///C:/Windows"), "#blocked-url");
    }

    #[test]
    fn test_sanitize_url_with_whitespace() {
        assert_eq!(sanitize_url("  javascript:alert(1)  "), "#blocked-url");
        assert_eq!(sanitize_url("\tdata:text/html,test\n"), "#blocked-url");
        assert_eq!(
            sanitize_url("  https://example.com  "),
            "  https://example.com  "
        );
    }

    #[test]
    fn test_sanitize_url_removes_disallowed_blank_like_chars() {
        assert_eq!(
            sanitize_url("https://exa\u{200B}mple.com/\u{3164}path"),
            "https://example.com/path"
        );
    }

    #[test]
    fn test_sanitize_url_blocks_scheme_after_normalization() {
        assert_eq!(sanitize_url("java\u{200B}script:alert(1)"), "#blocked-url");
        assert_eq!(sanitize_url("data:\u{FEFF}text/html,test"), "#blocked-url");
    }
}
