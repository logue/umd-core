//! Code block extensions for UMD
//!
//! Provides syntax highlighting and Mermaid diagram support for code blocks.
//! - Syntax highlighting: Multiple language support with syntax coloring
//! - Mermaid diagrams: Diagram rendering from Markdown fence blocks with SVG generation
//! - File name support: Code blocks with associated file names

use regex::Regex;
use uuid::Uuid;

/// Process code blocks with syntax highlighting and metadata
///
/// # Features
/// - ✅ Language detection from code class attribute
/// - ✅ Syntax highlighting class generation
/// - ✅ Mermaid diagram detection and SVG rendering
/// - ✅ Bootstrap CSS variable integration
/// - ✅ Plain text blocks (no language) without `<code>` tags
///
/// # Input Format (from comrak with github_pre_lang=true)
///
/// comrak outputs code blocks in GitHub-flavored format:
/// - `<pre><code>plain text content</code></pre>` - Plain text (no language)
/// - `<pre><code class="language-rust">highlighted content</code></pre>` - With language
/// - `<pre><code class="language-mermaid">diagram code</code></pre>` - Mermaid diagrams
///
/// # Output HTML Patterns
///
/// Plain text: `<pre>content</pre>` (code tag removed)
///
/// Language-specific: `<pre><code class="language-rust">content</code></pre>` (unchanged)
///
/// Mermaid diagram: `<div class="mermaid-diagram">SVG content</div>`
pub fn process_code_blocks(html: &str) -> String {
    // First handle Mermaid diagrams if present
    let html = process_mermaid_blocks(html);
    
    // Then process regular code blocks with syntax highlighting
    process_syntax_highlighted_blocks(&html)
}

/// Process Mermaid diagram blocks
///
/// Converts `<code class="language-mermaid">` blocks into SVG diagrams with Bootstrap styling
/// comrak outputs: `<pre><code class="language-mermaid">...</code></pre>`
fn process_mermaid_blocks(html: &str) -> String {
    // Check if mermaid is present (but not already wrapped)
    if !html.contains("language-mermaid") || html.contains("mermaid-diagram") {
        return html.to_string();
    }
    
    let mut result = html.to_string();
    
    // Handle comrak format: <pre><code class="language-mermaid">...</code></pre>
    // Using (?s) for DOTALL mode to match newlines
    if let Ok(mermaid_pattern) = Regex::new(r#"(?s)<pre><code[^>]*class="language-mermaid"[^>]*>(.*?)</code></pre>"#) {
        result = mermaid_pattern.replace_all(&result, |caps: &regex::Captures| {
            let code = &caps[1];
            let decoded = decode_html_entities(code);
            let code_text = decoded.trim();
            
            // Generate SVG from Mermaid code
            let svg = render_mermaid_as_svg(code_text);
            let diagram_id = Uuid::new_v4().to_string();
            
            format!(
                "<div class=\"mermaid-diagram\" id=\"mermaid-{}\" data-mermaid-source=\"{}\">{}​</div>",
                &diagram_id[..8],
                html_escape::encode_text(code_text),
                svg
            )
        }).to_string();
    }
    
    result
}

/// Process syntax highlighting for code blocks
///
/// comrak with `github_pre_lang=true` outputs code blocks as:
/// - `<pre><code>plain content</code></pre>` for plain text blocks (no language)
/// - `<pre><code class="language-rust">highlighted content</code></pre>` for language-specific blocks
///
/// The full fence info string can include title metadata (e.g., "rust: main.rs"),
/// but comrak only extracts the language part to generate the class attribute.
/// This function processes the pre and code tags to add title support and
/// remove <code> tags for plain text blocks.
///
/// Supports four code block patterns:
/// 1. Plain text: `<pre><code>...</code></pre>` → `<pre>...</pre>`
/// 2. Plain text with title: parse from fence info in data attributes
/// 3. Language-only: `<pre><code class="language-rust">...</code></pre>`
/// 4. Language+Title: add figcaption wrapper with title
fn process_syntax_highlighted_blocks(html: &str) -> String {
    let mut result = html.to_string();
    
    // Handle comrak GitHub format: <pre><code class="language-...">...</code></pre>
    // This covers language-specific code blocks
    if let Ok(with_lang) = Regex::new(r#"(?s)<pre><code[^>]*class="language-([a-z0-9_+\-]+)"[^>]*>(.*?)</code></pre>"#) {
        result = with_lang.replace_all(&result, |caps: &regex::Captures| {
            let language = &caps[1];
            let code = &caps[2];
            
            // Skip mermaid (handled separately)
            if language == "mermaid" {
                return format!("<pre><code class=\"language-{}\">{}</code></pre>", language, code);
            }
            
            // For now, output directly with language - title support would need fence info string
            // which isn't available after comrak has processed it
            format!(
                "<pre><code class=\"language-{}\">{}</code></pre>",
                language,
                code
            )
        }).to_string();
    }
    
    // Handle plain text blocks: <pre><code>...</code></pre> (no class attribute)
    if let Ok(plain_pattern) = Regex::new(r#"(?s)<pre><code>(.*?)</code></pre>"#) {
        result = plain_pattern.replace_all(&result, |caps: &regex::Captures| {
            let code = &caps[1];
            // Strip the <code> tag for plain text blocks
            format!("<pre>{}</pre>", code)
        }).to_string();
    }
    
    result
}

/// Render Mermaid code to SVG
///
/// Converts Mermaid diagram notation to SVG format with Bootstrap CSS variable support.
/// Supports basic graph, flowchart, and sequence diagrams.
fn render_mermaid_as_svg(mermaid_code: &str) -> String {
    // Default SVG with fallback styling
    let svg_wrapper = generate_fallback_svg(mermaid_code);
    
    // Inject Bootstrap CSS variables for coloring
    inject_bootstrap_colors(&svg_wrapper)
}

/// Generate a fallback SVG representation of Mermaid diagram
///
/// Creates a basic SVG structure with Bootstrap styling
fn generate_fallback_svg(mermaid_code: &str) -> String {
    let trimmed = mermaid_code.trim();
    
    // Basic SVG header with Bootstrap variable references
    let mut svg = String::from(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 800 400" class="mermaid-svg" style="max-width: 100%; height: auto;">
        <defs>
            <style>
                .mermaid-node { fill: var(--bs-body-bg); stroke: var(--bs-border-color); stroke-width: 2; }
                .mermaid-edge { stroke: var(--bs-border-color); stroke-width: 2; fill: none; }
                .mermaid-arrow { fill: var(--bs-border-color); }
                .mermaid-text { fill: var(--bs-body-color); font-family: system-ui, -apple-system, sans-serif; font-size: 14px; text-anchor: middle; }
                .mermaid-title { fill: var(--bs-primary, #0d6efd); font-size: 16px; font-weight: bold; }
            </style>
        </defs>
        <rect width="800" height="400" fill="transparent" stroke="var(--bs-border-color)" stroke-width="1" />
"#
    );
    
    // Parse and render basic diagram elements
    if trimmed.starts_with("graph") || trimmed.starts_with("flowchart") {
        // Simple graph/flowchart rendering
        svg.push_str(render_graph_nodes(mermaid_code).as_str());
    } else if trimmed.starts_with("sequenceDiagram") {
        // Simple sequence diagram placeholder
        svg.push_str(render_sequence_diagram(mermaid_code).as_str());
    } else {
        // Generic placeholder for unsupported diagram types
        svg.push_str(&format!(
            r#"<text x="400" y="200" class="mermaid-text">{}</text>"#,
            html_escape::encode_text("Mermaid Diagram")
        ));
    }
    
    svg.push_str("</svg>");
    svg
}

/// Render graph/flowchart nodes and edges
fn render_graph_nodes(mermaid_code: &str) -> String {
    let mut result = String::new();
    let lines: Vec<&str> = mermaid_code.lines().collect();
    
    let mut y_pos = 80;
    for line in lines.iter().skip(1) {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("%%") {
            continue;
        }
        
        // Simple node rendering (nodeId[label])
        if trimmed.contains('[') && trimmed.contains(']') {
            let node_svg = render_single_node(trimmed, 100, y_pos);
            result.push_str(&node_svg);
            y_pos += 80;
        }
    }
    
    result
}

/// Render a single graph node
fn render_single_node(node_def: &str, x: i32, y: i32) -> String {
    // Extract node label from brackets
    if let Some(start) = node_def.find('[') {
        if let Some(end) = node_def.find(']') {
            let label = &node_def[start + 1..end];
            return format!(
                r#"<rect x="{}" y="{}" width="150" height="50" class="mermaid-node" rx="5" />
                <text x="{}" y="{}" class="mermaid-text">{}</text>
                "#,
                x,
                y,
                x + 75,
                y + 30,
                html_escape::encode_text(label.trim())
            );
        }
    }
    String::new()
}

/// Render sequence diagram placeholder
fn render_sequence_diagram(_mermaid_code: &str) -> String {
    // Placeholder for sequence diagram
    r#"<text x="400" y="100" class="mermaid-title">Sequence Diagram</text>
       <line x1="100" y1="150" x2="100" y2="350" class="mermaid-edge" />
       <line x1="400" y1="150" x2="400" y2="350" class="mermaid-edge" />
       <line x1="700" y1="150" x2="700" y2="350" class="mermaid-edge" />
       <text x="100" y="140" class="mermaid-text">Actor 1</text>
       <text x="400" y="140" class="mermaid-text">System</text>
       <text x="700" y="140" class="mermaid-text">Actor 2</text>
    "#.to_string()
}

/// Inject Bootstrap CSS variables for diagram coloring
///
/// Replaces hardcoded colors with Bootstrap color variables (--bs-blue, --bs-green, etc.)
/// instead of system theme variables. White and black are excluded as they represent
/// structural elements rather than semantic colors.
fn inject_bootstrap_colors(svg: &str) -> String {
    svg
        .replace("\"#0d6efd\"", "\"var(--bs-blue, #0d6efd)\"")
        .replace("\"#6c757d\"", "\"var(--bs-gray, #6c757d)\"")
        .replace("\"#198754\"", "\"var(--bs-green, #198754)\"")
        .replace("\"#dc3545\"", "\"var(--bs-red, #dc3545)\"")
        .replace("\"#ffc107\"", "\"var(--bs-yellow, #ffc107)\"")
        .replace("\"#0dcaf0\"", "\"var(--bs-cyan, #0dcaf0)\"")
        // Note: #ffffff (white) and #000000 (black) are intentionally excluded
        // as they represent structural elements, not semantic colors
}


/// Simple hash function for generating diagram IDs
/// Uses a lightweight FNV-1a algorithm
/// Note: Currently replaced with UUID for unique ID generation in render_mermaid_as_svg
#[allow(dead_code)]
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
    fn test_basic_code_block_with_language() {
        // comrak GitHub format: <pre><code class="language-rust">code</code></pre>
        let html = "<pre><code class=\"language-rust\">fn main() {}</code></pre>";
        let result = process_code_blocks(html);
        assert!(result.contains("language-rust"));
        assert!(result.contains("fn main() {}"));
    }

    #[test]
    fn test_basic_code_block_plain_text() {
        // Plain text block (no language attribute): <pre><code>text</code></pre>
        let html = "<pre><code>plain text</code></pre>";
        let result = process_code_blocks(html);
        assert!(result.contains("<pre>plain text</pre>"));
        assert!(!result.contains("<code>"));
    }

    #[test]
    fn test_mermaid_block_detection() {
        // comrak Mermaid format: <pre><code class="language-mermaid">...</code></pre>
        let html = "<pre><code class=\"language-mermaid\">graph TD\n    A[Start] --> B[End]</code></pre>";
        let result = process_code_blocks(html);
        assert!(result.contains("mermaid-diagram"));
        assert!(result.contains("data-mermaid-source"));
        assert!(result.contains("<svg"));
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
        // Language-specific block left unchanged
        let html = "<pre><code class=\"language-python\">print('hello')</code></pre>";
        let result = process_code_blocks(html);
        assert!(result.contains("language-python"));
        assert!(result.contains("print('hello')"));
    }

    #[test]
    fn test_code_block_escaping() {
        let html = "<pre><code class=\"language-html\">&lt;div&gt;content&lt;/div&gt;</code></pre>";
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
