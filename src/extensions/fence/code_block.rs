//! Code block extensions for UMD
//!
//! Handles fenced code block metadata: figure/figcaption wrapping and
//! filename support. Syntax highlighting and any language-specific
//! rendering (e.g. Mermaid diagrams, GeoJSON maps) are intentionally
//! *not* performed here — that is host application territory. umd-core
//! only preserves comrak's `language-*` class on the `<code>` element so
//! the host can detect it and enhance the block itself. See
//! docs/architecture.md's 3-layer deferred-rendering model.

use once_cell::sync::Lazy;
use regex::Regex;

static CODE_BLOCK_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?s)<pre><code(?P<attrs>[^>]*)>(?P<code>.*?)</code></pre>"#)
        .expect("valid code block regex")
});

static HTML_ATTR_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"([a-zA-Z_:][-a-zA-Z0-9_:.]*)\s*=\s*\"([^\"]*)\""#).expect("valid html attr regex")
});

/// Process code blocks: figure/figcaption wrapping and filename metadata
///
/// # Features
/// - ✅ Language class passthrough (no interpretation, no highlighting)
/// - ✅ Filename metadata → `<figcaption>`
/// - ✅ Plain text blocks (no language) without `<code>` tags
///
/// # Input Format (from comrak)
///
/// comrak outputs code blocks in GitHub-flavored format:
/// - `<pre><code>plain text content</code></pre>` - Plain text (no language)
/// - `<pre><code class="language-rust">content</code></pre>` - With language
///
/// # Output HTML Patterns
///
/// Plain text: `<pre>content</pre>` (code tag removed)
///
/// Language-specific: `<pre><code class="language-rust">content</code></pre>` (unchanged)
///
/// Every code block is wrapped in `<figure class="umd-code-block">`. A
/// language such as `mermaid` or `geojson` is passed through exactly like
/// any other language umd-core doesn't recognize — the host application is
/// expected to detect `language-mermaid`/`language-geojson` etc. on the
/// `<code>` element and replace/enhance the block client-side or at build
/// time (see docs/architecture.md).
pub fn process_code_blocks(html: &str) -> String {
    process_syntax_highlighted_blocks(html)
}

/// Process code blocks: add figure/figcaption wrapping and filename metadata
///
/// comrak outputs code blocks as:
/// - `<pre><code>plain content</code></pre>` for plain text blocks (no language)
/// - `<pre><code class="language-rust">content</code></pre>` for language-specific blocks
///
/// The full fence info string can include title metadata (e.g., "rust: main.rs"),
/// but comrak only extracts the language part to generate the class attribute.
/// This function processes the pre and code tags to add title support and
/// remove <code> tags for plain text blocks.
///
/// Supports four code block patterns:
/// 1. Plain text: `<pre><code>...</code></pre>` → `<pre>...</pre>`
/// 2. Plain text with title: parse from fence info in data attributes
/// 3. Language-only: `<pre><code class="language-rust">...</code></pre>` (unchanged)
/// 4. Language+Title: add figcaption wrapper with title
fn process_syntax_highlighted_blocks(html: &str) -> String {
    CODE_BLOCK_RE
        .replace_all(html, |caps: &regex::Captures| {
            let attrs = caps.name("attrs").map(|m| m.as_str()).unwrap_or("");
            let code = caps.name("code").map(|m| m.as_str()).unwrap_or("");

            let language = extract_language_from_attrs(attrs);

            let filename = extract_attribute(attrs, "data-meta")
                .map(|value| decode_html_entities(&value))
                .and_then(|meta| extract_filename_from_meta(&meta));

            let rendered_block = if let Some(lang) = language.as_deref() {
                format!("<pre><code class=\"language-{}\">{}</code></pre>", lang, code)
            } else {
                format!("<pre>{}</pre>", code)
            };

            // Always wrap in `<figure class="umd-code-block">`, with or
            // without a filename, so every code block interoperates with the
            // START:/CENTER:/END:/JUSTIFY: placement decorators the same way
            // tables and media do (see alignment::apply_pending_code_block_placement).
            let figcaption = filename
                .map(|filename| {
                    format!(
                        "<figcaption class=\"umd-code-filename\">{}</figcaption>",
                        html_escape::encode_text(&filename)
                    )
                })
                .unwrap_or_default();

            format!(
                "<figure class=\"umd-code-block\">{}{}</figure>",
                figcaption, rendered_block
            )
        })
        .to_string()
}

fn extract_attribute(attrs: &str, name: &str) -> Option<String> {
    for caps in HTML_ATTR_RE.captures_iter(attrs) {
        if caps.get(1)?.as_str().eq_ignore_ascii_case(name) {
            return Some(caps.get(2)?.as_str().to_string());
        }
    }
    None
}

fn extract_language_from_attrs(attrs: &str) -> Option<String> {
    let class_attr = extract_attribute(attrs, "class")?;

    for class_name in class_attr.split_whitespace() {
        if let Some(language) = class_name.strip_prefix("language-") {
            if language.eq_ignore_ascii_case("umd-nolang") {
                return None;
            }
            if !language.is_empty() {
                return Some(language.to_string());
            }
        }
    }

    None
}

fn extract_filename_from_meta(meta: &str) -> Option<String> {
    let marker = "umd-filename:";
    let index = meta.find(marker)?;
    let filename = meta[index + marker.len()..].trim();
    if filename.is_empty() {
        None
    } else {
        Some(filename.to_string())
    }
}

/// Basic HTML entity decoder for common entities
fn decode_html_entities(s: &str) -> String {
    s.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_code_block_with_language() {
        // comrak GitHub format: <pre><code class="language-rust">code</code></pre>
        let html = "<pre><code class=\"language-rust\">fn main() {}</code></pre>";
        let result = process_code_blocks(html);
        assert!(result.contains("language-rust"));
        assert!(result.contains("umd-code-block"));
        assert!(result.contains("fn"));
        assert!(result.contains("main"));
    }

    #[test]
    fn test_basic_code_block_plain_text() {
        // Plain text block (no language attribute): <pre><code>text</code></pre>
        let html = "<pre><code>plain text</code></pre>";
        let result = process_code_blocks(html);
        assert_eq!(
            result,
            "<figure class=\"umd-code-block\"><pre>plain text</pre></figure>"
        );
    }

    #[test]
    fn test_mermaid_block_passthrough() {
        // Mermaid (like any other language umd-core doesn't specially
        // handle) passes through unchanged as a plain language-tagged code
        // block; rendering it is the host application's responsibility.
        let html =
            "<pre><code class=\"language-mermaid\">graph TD\n    A[Start] --> B[End]</code></pre>";
        let result = process_code_blocks(html);
        assert!(result.contains("language-mermaid"));
        assert!(result.contains("umd-code-block"));
        assert!(!result.contains("<svg"));
    }

    #[test]
    fn test_code_block_plain_text_no_code_tag() {
        // Plain text: <pre><code>...</code></pre> → <pre>...</pre>
        let html = "<pre><code>plain text here</code></pre>";
        let result = process_code_blocks(html);
        assert!(result.contains("<pre>plain text here</pre>"));
        assert!(!result.contains("<code>"));
    }

    #[test]
    fn test_code_block_multiline_plain_text() {
        // Plain text block with newlines
        let html = "<pre><code>line1\nline2\nline3</code></pre>";
        let result = process_code_blocks(html);
        assert!(result.contains("<pre>line1\nline2\nline3</pre>"));
        assert!(!result.contains("<code>"));
    }

    #[test]
    fn test_code_block_language_preserved() {
        // Language-specific block left unchanged, no highlighting applied
        let html = "<pre><code class=\"language-python\">print('hello')</code></pre>";
        let result = process_code_blocks(html);
        assert!(result.contains("language-python"));
        assert!(result.contains("print"));
        assert!(result.contains("hello"));
    }

    #[test]
    fn test_code_block_escaping() {
        let html = "<pre><code class=\"language-html\">&lt;div&gt;content&lt;/div&gt;</code></pre>";
        let result = process_code_blocks(html);
        assert!(result.contains("&lt;"));
        assert!(result.contains("&gt;"));
        assert!(result.contains("content"));
    }

    #[test]
    fn test_decoded_html_entities() {
        let encoded = "&lt;div&gt; &amp; &quot;test&quot;";
        let decoded = decode_html_entities(encoded);
        assert_eq!(decoded, "<div> & \"test\"");
    }

    #[test]
    fn test_code_block_with_filename_and_language() {
        let html = "<pre><code class=\"language-rust\" data-meta=\"umd-filename:src/main.rs\">fn main() {}</code></pre>";
        let result = process_code_blocks(html);
        assert!(result.contains("<figure class=\"umd-code-block\">"));
        assert!(result.contains("<figcaption class=\"umd-code-filename\">"));
        assert!(result.contains("src/main.rs"));
        assert!(result.contains("language-rust"));
    }

    #[test]
    fn test_code_block_with_filename_without_language() {
        let html = "<pre><code class=\"language-umd-nolang\" data-meta=\"umd-filename:config.yml\">key: value</code></pre>";
        let result = process_code_blocks(html);
        assert!(result.contains("<figure class=\"umd-code-block\">"));
        assert!(result.contains("config.yml"));
        assert!(result.contains("<pre>key: value</pre>"));
        assert!(!result.contains("language-umd-nolang"));
    }
}
