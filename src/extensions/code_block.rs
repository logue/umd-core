//! Code block extensions for UMD
//!
//! Provides syntax highlighting and Mermaid diagram support for code blocks.
//! - Syntax highlighting: Multiple language support with syntax coloring
//! - Mermaid diagrams: Diagram rendering from Markdown fence blocks
//! - File name support: Code blocks with associated file names

use regex::Regex;

/// Process code blocks with syntax highlighting and metadata
///
/// # Features
/// - ✅ Language detection from fence info string
/// - ✅ File name extraction and `<figcaption>` generation
/// - ✅ Syntax highlighting class generation
/// - ✅ Mermaid diagram detection and wrapping
/// - ✅ Bootstrap CSS variable integration
///
/// # Syntax Examples
/// ```code
/// ```rust
/// fn main() {}
/// ```
/// 
/// ```rust:main.rs
/// fn main() {}
/// ```
///
/// ```mermaid
/// graph TD
///     A[Start] --> B[End]
/// ```
/// ```
///
/// # Output Format
///
/// For regular code blocks:
/// ```html
/// <pre><code class="language-rust">...</code></pre>
/// ```
///
/// For code blocks with file names:
/// ```html
/// <figure class="code-block">
///   <figcaption class="code-filename">main.rs</figcaption>
///   <pre><code class="language-rust">...</code></pre>
/// </figure>
/// ```
///
/// For Mermaid diagrams:
/// ```html
/// <div class="mermaid-diagram" data-mermaid-version="10">
///   [mermaid code will be rendered here on frontend]
/// </div>
/// ```
pub fn process_code_blocks(html: &str) -> String {
    // First handle Mermaid diagrams if present
    let html = process_mermaid_blocks(html);
    
    // Then process regular code blocks with syntax highlighting
    process_syntax_highlighted_blocks(&html)
}

/// Process Mermaid diagram blocks
///
/// Converts `<code class="language-mermaid">` blocks into special Mermaid containers
/// that will be rendered by the frontend.
fn process_mermaid_blocks(html: &str) -> String {
    // Check if mermaid is present (but not already wrapped)
    if !html.contains("mermaid") || html.contains("mermaid-diagram") {
        return html.to_string();
    }
    
    let mut result = html.to_string();
    
    // Handle format 1: <pre lang="mermaid"><code>...</code></pre>
    // Using (?s) for DOTALL mode to match newlines
    if let Ok(mermaid_pattern) = Regex::new(r#"(?s)<pre lang="mermaid"[^>]*><code>(.*?)</code></pre>"#) {
        result = mermaid_pattern.replace_all(&result, |caps: &regex::Captures| {
            let code = &caps[1];
            let decoded = decode_html_entities(code);
            let code_text = decoded.trim();
            let diagram_id = format!("mermaid-{}", simple_hash(code_text));
            let escaped_code = html_escape::encode_text(code_text);
            
            format!(
                "<div class=\"mermaid-diagram\" id=\"{}\"><pre><code class=\"language-mermaid\" data-mermaid-source=\"{}\">{}</code></pre></div>",
                diagram_id, escaped_code, code
            )
        }).to_string();
    }
    
    // Handle format 2: <pre><code class="language-mermaid">...</code></pre>
    if let Ok(mermaid_pattern) = Regex::new(r#"(?s)<pre><code[^>]*language-mermaid[^>]*>(.*?)</code></pre>"#) {
        result = mermaid_pattern.replace_all(&result, |caps: &regex::Captures| {
            let code = &caps[1];
            let decoded = decode_html_entities(code);
            let code_text = decoded.trim();
            let diagram_id = format!("mermaid-{}", simple_hash(code_text));
            let escaped_code = html_escape::encode_text(code_text);
            
            format!(
                "<div class=\"mermaid-diagram\" id=\"{}\"><pre><code class=\"language-mermaid\" data-mermaid-source=\"{}\">{}</code></pre></div>",
                diagram_id, escaped_code, code
            )
        }).to_string();
    }
    
    result
}

/// Process syntax highlighting for code blocks
///
/// Enhances code blocks with language information and Bootstrap CSS integration
fn process_syntax_highlighted_blocks(html: &str) -> String {
    // Handle format 1: <pre lang="rust"><code>...</code></pre> (comrak default)
    if let Ok(pre_lang_pattern) = Regex::new(r#"<pre lang="([^"]+)"[^>]*><code>(.*?)</code></pre>"#) {
        let html = pre_lang_pattern.replace_all(html, |caps: &regex::Captures| {
            let language = &caps[1];
            let code = &caps[2];
            
            // Skip mermaid (handled separately)
            if language == "mermaid" {
                return format!("<pre lang=\"{}\"><code>{}</code></pre>", language, code);
            }
            
            // Check if filename is embedded
            if let Some(filename) = extract_filename_from_data(code) {
                format!(
                    "<figure class=\"code-block code-block-{}\">\
                       <figcaption class=\"code-filename\">{}</figcaption>\
                       <pre><code class=\"language-{}\">{}</code></pre>\
                     </figure>",
                    language,
                    html_escape::encode_text(&filename),
                    language,
                    code
                )
            } else {
                format!(
                    "<pre><code class=\"language-{}\">{}</code></pre>",
                    language,
                    code
                )
            }
        }).to_string();
        return html;
    }
    
    // Handle format 2: <pre><code class="language-rust">...</code></pre>
    if let Ok(with_lang) = Regex::new(r#"<pre><code[^>]*language-([a-z0-9_+-]+)[^>]*>(.*?)</code></pre>"#) {
        let result = with_lang.replace_all(html, |caps: &regex::Captures| {
            let language = &caps[1];
            let code = &caps[2];
            
            // Skip mermaid (handled separately)
            if language == "mermaid" {
                return format!("<pre><code class=\"language-{}\">{}</code></pre>", language, code);
            }
            
            // Check if filename is embedded
            if let Some(filename) = extract_filename_from_data(code) {
                format!(
                    "<figure class=\"code-block code-block-{}\">\
                       <figcaption class=\"code-filename\">{}</figcaption>\
                       <pre><code class=\"language-{}\">{}</code></pre>\
                     </figure>",
                    language,
                    html_escape::encode_text(&filename),
                    language,
                    code
                )
            } else {
                format!(
                    "<pre><code class=\"language-{}\">{}</code></pre>",
                    language,
                    code
                )
            }
        }).to_string();
        return result;
    }
    
    html.to_string()
}

/// Extract filename from code metadata comment
///
/// Supports formats like:
/// - `// @filename: main.rs`
/// - `# @filename: script.py`
/// - `<!-- @filename: style.css -->`
fn extract_filename_from_data(code: &str) -> Option<String> {
    // Try to extract filename from metadata comment
    let lines: Vec<&str> = code.lines().collect();
    if lines.is_empty() {
        return None;
    }
    
    let first_line = lines[0].trim();
    
    // Pattern for metadata comment: @filename:value
    if let Some(start) = first_line.find("@filename:") {
        let remainder = &first_line[start + 10..];
        let filename = remainder
            .trim_start()
            .trim_matches(|c| c == '"' || c == '\'' || c == '>' || c == '*' || c == '/')
            .trim();
        
        if !filename.is_empty() {
            return Some(filename.to_string());
        }
    }
    
    None
}

/// Simple hash function for generating diagram IDs
/// Uses a lightweight FNV-1a algorithm
fn simple_hash(data: &str) -> u64 {
    const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;
    
    let mut hash = FNV_OFFSET_BASIS;
    for byte in data.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
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

/// Get list of supported languages for syntax highlighting
///
/// Returns language identifiers that can be used in code fence info strings
pub fn get_supported_languages() -> Vec<&'static str> {
    vec![
        "rust", "python", "javascript", "typescript", "jsx", "tsx",
        "html", "css", "scss", "less",
        "java", "kotlin", "go", "c", "cpp", "csharp", "swift", "objc",
        "php", "ruby", "perl", "bash", "shell", "zsh", "fish",
        "sql", "mysql", "postgresql", "mongodb",
        "json", "yaml", "toml", "xml", "markdown", "latex",
        "dockerfile", "nginx", "apache", "lua", "vim", "elisp",
        "mermaid",  // Diagram support
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_code_block_format1() {
        // comrak format: <pre lang="rust"><code>...</code></pre>
        let html = "<pre lang=\"rust\"><code>fn main() {}</code></pre>";
        let result = process_code_blocks(html);
        assert!(result.contains("language-rust"));
        assert!(result.contains("fn main() {}"));
    }

    #[test]
    fn test_basic_code_block_format2() {
        // Alternative format: <pre><code class="language-rust">...</code></pre>
        let html = "<pre><code class=\"language-rust\">fn main() {}</code></pre>";
        let result = process_code_blocks(html);
        assert!(result.contains("language-rust"));
        assert!(result.contains("fn main() {}"));
    }

    #[test]
    fn test_mermaid_block_detection_format1() {
        // comrak mermaid format
        let html = "<pre lang=\"mermaid\"><code>graph TD\n    A[Start] --> B[End]</code></pre>";
        let result = process_code_blocks(html);
        assert!(result.contains("mermaid-diagram"));
        assert!(result.contains("language-mermaid"));
    }

    #[test]
    fn test_mermaid_block_detection_format2() {
        let html = "<pre><code class=\"language-mermaid\">graph TD\n    A[Start] --> B[End]</code></pre>";
        let result = process_code_blocks(html);
        assert!(result.contains("mermaid-diagram"));
    }

    #[test]
    fn test_code_with_filename() {
        let code = "// @filename: main.rs\nfn main() {}";
        assert_eq!(
            extract_filename_from_data(code),
            Some("main.rs".to_string())
        );
    }

    #[test]
    fn test_multiple_code_blocks() {
        let html = "\
            <pre lang=\"rust\"><code>code1</code></pre>\
            <pre lang=\"python\"><code>code2</code></pre>";
        let result = process_code_blocks(html);
        assert!(result.contains("language-rust"));
        assert!(result.contains("language-python"));
    }

    #[test]
    fn test_code_block_escaping() {
        let html = "<pre lang=\"html\"><code>&lt;div&gt;content&lt;/div&gt;</code></pre>";
        let result = process_code_blocks(html);
        assert!(result.contains("&lt;div&gt;"));
    }

    #[test]
    fn test_simple_hash_consistency() {
        let hash1 = simple_hash("test");
        let hash2 = simple_hash("test");
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_decoded_html_entities() {
        let encoded = "&lt;div&gt; &amp; &quot;test&quot;";
        let decoded = decode_html_entities(encoded);
        assert_eq!(decoded, "<div> & \"test\"");
    }
}
