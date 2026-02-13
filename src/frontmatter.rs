//! Frontmatter parsing module
//!
//! Supports YAML and TOML frontmatter extraction from wiki markup.
//! Frontmatter is metadata placed at the beginning of a document.

use once_cell::sync::Lazy;
use regex::Regex;

/// Supported frontmatter formats
#[derive(Debug, Clone, PartialEq)]
pub enum FrontmatterFormat {
    /// YAML format (delimited by ---)
    Yaml,
    /// TOML format (delimited by +++)
    Toml,
}

/// Extracted frontmatter data
#[derive(Debug, Clone)]
pub struct Frontmatter {
    /// The format of the frontmatter
    pub format: FrontmatterFormat,
    /// The raw frontmatter content (without delimiters)
    pub content: String,
}

static YAML_FRONTMATTER: Lazy<Regex> = Lazy::new(|| {
    // Match YAML frontmatter: ---\n...content...\n---
    Regex::new(r"^---\s*\n([\s\S]*?)\n---\s*\n").unwrap()
});

static TOML_FRONTMATTER: Lazy<Regex> = Lazy::new(|| {
    // Match TOML frontmatter: +++\n...content...\n+++
    Regex::new(r"^\+\+\+\s*\n([\s\S]*?)\n\+\+\+\s*\n").unwrap()
});

/// Extract frontmatter from input text
///
/// Checks for YAML or TOML frontmatter at the beginning of the text.
/// If found, returns the frontmatter data and the remaining content.
///
/// # Arguments
///
/// * `input` - The input text that may contain frontmatter
///
/// # Returns
///
/// A tuple of (optional frontmatter, remaining content)
///
/// # Examples
///
/// ```
/// use umd::frontmatter::extract_frontmatter;
///
/// let input = "---\ntitle: Hello\nauthor: John\n---\n\n# Content";
/// let (frontmatter, content) = extract_frontmatter(input);
/// assert!(frontmatter.is_some());
/// assert!(content.contains("# Content"));
/// ```
pub fn extract_frontmatter(input: &str) -> (Option<Frontmatter>, String) {
    // Try YAML first
    if let Some(caps) = YAML_FRONTMATTER.captures(input) {
        let fm_content = caps.get(1).map_or("", |m| m.as_str());
        let remaining = YAML_FRONTMATTER.replace(input, "").to_string();

        return (
            Some(Frontmatter {
                format: FrontmatterFormat::Yaml,
                content: fm_content.to_string(),
            }),
            remaining,
        );
    }

    // Try TOML
    if let Some(caps) = TOML_FRONTMATTER.captures(input) {
        let fm_content = caps.get(1).map_or("", |m| m.as_str());
        let remaining = TOML_FRONTMATTER.replace(input, "").to_string();

        return (
            Some(Frontmatter {
                format: FrontmatterFormat::Toml,
                content: fm_content.to_string(),
            }),
            remaining,
        );
    }

    // No frontmatter found
    (None, input.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_yaml_frontmatter() {
        let input = "---\ntitle: Test\nauthor: John\n---\n\n# Content";
        let (fm, content) = extract_frontmatter(input);

        assert!(fm.is_some());
        let fm = fm.unwrap();
        assert_eq!(fm.format, FrontmatterFormat::Yaml);
        assert!(fm.content.contains("title: Test"));
        assert!(content.contains("# Content"));
        assert!(!content.contains("---"));
    }

    #[test]
    fn test_toml_frontmatter() {
        let input = "+++\ntitle = \"Test\"\nauthor = \"John\"\n+++\n\n# Content";
        let (fm, content) = extract_frontmatter(input);

        assert!(fm.is_some());
        let fm = fm.unwrap();
        assert_eq!(fm.format, FrontmatterFormat::Toml);
        assert!(fm.content.contains("title = \"Test\""));
        assert!(content.contains("# Content"));
        assert!(!content.contains("+++"));
    }

    #[test]
    fn test_no_frontmatter() {
        let input = "# Just a heading\n\nSome content";
        let (fm, content) = extract_frontmatter(input);

        assert!(fm.is_none());
        assert_eq!(content, input);
    }

    #[test]
    fn test_yaml_with_complex_content() {
        let input = "---\ntitle: Complex\ntags:\n  - rust\n  - wiki\ndate: 2024-01-01\n---\n\n**Bold** text";
        let (fm, content) = extract_frontmatter(input);

        assert!(fm.is_some());
        let fm = fm.unwrap();
        assert!(fm.content.contains("tags:"));
        assert!(content.contains("**Bold**"));
    }

    #[test]
    fn test_frontmatter_must_be_at_start() {
        let input = "Some text\n---\ntitle: Test\n---\n\nMore content";
        let (fm, content) = extract_frontmatter(input);

        // Should not detect frontmatter if not at the beginning
        assert!(fm.is_none());
        assert_eq!(content, input);
    }
}
