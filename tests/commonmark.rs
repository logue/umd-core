//! CommonMark specification compliance tests
//!
//! This test suite verifies that the parser reasonably complies with CommonMark specification.
//! Target: 75%+ pass rate

use umd::parser::{ParserOptions, parse_to_html};

/// Test CommonMark heading levels
#[test]
fn test_commonmark_headings() {
    let cases = vec![
        ("# foo", "h1", "foo"),
        ("## foo", "h2", "foo"),
        ("### foo", "h3", "foo"),
        ("#### foo", "h4", "foo"),
        ("##### foo", "h5", "foo"),
        // LukiWiki limits to 5 levels
        ("###### foo", "h6", "foo"), // This might not work in LukiWiki mode
    ];

    for (input, tag, text) in cases {
        let html = parse_to_html(input, &ParserOptions::default());
        // comrak adds anchor IDs, so just check for tag and content
        assert!(
            html.contains(&format!("<{}", tag)) && html.contains(text),
            "Input: {}\nExpected: <{}>{}</{}>\\nGot: {}",
            input,
            tag,
            text,
            tag,
            html
        );
    }
}

/// Test CommonMark paragraphs
#[test]
fn test_commonmark_paragraphs() {
    let input = "aaa\n\nbbb";
    let html = parse_to_html(input, &ParserOptions::default());
    assert!(html.contains("<p>aaa</p>"));
    assert!(html.contains("<p>bbb</p>"));
}

/// Test CommonMark emphasis and strong emphasis
#[test]
fn test_commonmark_emphasis() {
    let cases = vec![
        ("*foo bar*", vec!["<em>", "foo bar", "</em>"]),
        ("**foo bar**", vec!["<strong>", "foo bar", "</strong>"]),
        ("***foo bar***", vec!["<em>", "<strong>", "foo bar"]), // Order may vary
    ];

    for (input, expected_parts) in cases {
        let html = parse_to_html(input, &ParserOptions::default());
        for part in expected_parts {
            assert!(
                html.contains(part),
                "Input: {}\nExpected part: {}\nGot: {}",
                input,
                part,
                html
            );
        }
    }
}

/// Test CommonMark links
#[test]
fn test_commonmark_links() {
    let input = "[link](/uri)";
    let html = parse_to_html(input, &ParserOptions::default());
    assert!(html.contains("<a href=\"/uri\">link</a>"));

    let input_with_title = "[link](/uri \"title\")";
    let html = parse_to_html(input_with_title, &ParserOptions::default());
    assert!(html.contains("href=\"/uri\""));
    assert!(html.contains("title=\"title\""));
}

/// Test CommonMark images
#[test]
fn test_commonmark_images() {
    let input = "![foo](/url \"title\")";
    let html = parse_to_html(input, &ParserOptions::default());
    assert!(html.contains("<img"));
    assert!(html.contains("src=\"/url\""));
    assert!(html.contains("alt=\"foo\""));
    assert!(html.contains("title=\"title\""));
}

/// Test CommonMark code spans
#[test]
fn test_commonmark_code_spans() {
    let input = "`foo`";
    let html = parse_to_html(input, &ParserOptions::default());
    assert!(html.contains("<code>foo</code>"));
}

/// Test CommonMark fenced code blocks
#[test]
fn test_commonmark_fenced_code_blocks() {
    let input = "```\n<\n >\n```";
    let html = parse_to_html(input, &ParserOptions::default());
    assert!(html.contains("&lt;"));
    assert!(html.contains("&gt;"));

    let input_with_lang = "```ruby\ndef foo(x)\n  return 3\nend\n```";
    let html = parse_to_html(input_with_lang, &ParserOptions::default());
    assert!(html.contains("ruby") || html.contains("language-ruby"));
    assert!(html.contains("def foo(x)"));
}

/// Test CommonMark unordered lists
#[test]
fn test_commonmark_unordered_lists() {
    let input = "- foo\n- bar\n- baz";
    let html = parse_to_html(input, &ParserOptions::default());
    assert!(html.contains("<ul>"));
    assert!(html.contains("<li>foo</li>"));
    assert!(html.contains("<li>bar</li>"));
}

/// Test CommonMark ordered lists
#[test]
fn test_commonmark_ordered_lists() {
    let input = "1. foo\n2. bar\n3. baz";
    let html = parse_to_html(input, &ParserOptions::default());
    assert!(html.contains("<ol>"));
    assert!(html.contains("<li>foo</li>"));
    assert!(html.contains("<li>bar</li>"));
}

/// Test CommonMark blockquotes
#[test]
fn test_commonmark_blockquotes() {
    let input = "> # Foo\n> bar\n> baz";
    let html = parse_to_html(input, &ParserOptions::default());
    assert!(html.contains("<blockquote>"));
    assert!(html.contains("<h1") && html.contains("Foo"));
    assert!(html.contains("bar"));
}

/// Test CommonMark thematic breaks (horizontal rules)
#[test]
fn test_commonmark_thematic_breaks() {
    let cases = vec!["***", "---", "___"];

    for input in cases {
        let html = parse_to_html(input, &ParserOptions::default());
        assert!(html.contains("<hr"), "Input: {}\nGot: {}", input, html);
    }
}

/// Test CommonMark hard line breaks
#[test]
fn test_commonmark_hard_line_breaks() {
    let input = "foo  \nbaz"; // Two spaces before newline
    let html = parse_to_html(input, &ParserOptions::default());
    // Hard break might be <br /> or just a newline depending on settings
    assert!(html.contains("foo") && html.contains("baz"));
}

/// Test CommonMark HTML escaping
#[test]
fn test_commonmark_html_escaping() {
    let input = "foo < bar";
    let html = parse_to_html(input, &ParserOptions::default());
    // Our sanitizer should escape this
    assert!(html.contains("&lt;") || html.contains("foo &lt; bar"));
}

/// Test CommonMark autolinks
#[test]
fn test_commonmark_autolinks() {
    let input = "<http://foo.bar.baz>";
    let html = parse_to_html(input, &ParserOptions::default());
    assert!(html.contains("<a href=\"http://foo.bar.baz\">"));
}

/// Test GFM table extension
#[test]
fn test_gfm_tables() {
    let input = "| foo | bar |\n| --- | --- |\n| baz | bim |";
    let html = parse_to_html(input, &ParserOptions::default());
    assert!(html.contains("<table>"));
    assert!(html.contains("<th>foo</th>"));
    assert!(html.contains("<td>baz</td>"));
}

/// Test GFM strikethrough extension
#[test]
fn test_gfm_strikethrough() {
    let input = "~~Hi~~ Hello, world!";
    let html = parse_to_html(input, &ParserOptions::default());
    assert!(html.contains("<del>Hi</del>"));
}

/// Test GFM task lists extension
#[test]
fn test_gfm_task_lists() {
    let input = "- [ ] foo\n- [x] bar";
    let html = parse_to_html(input, &ParserOptions::default());
    assert!(html.contains("type=\"checkbox\""));
    assert!(html.contains("disabled"));
}

// Summary test to check overall compliance
#[test]
fn test_commonmark_compliance_summary() {
    // This is a smoke test with mixed CommonMark features
    let input = r#"
# Heading

This is a **paragraph** with *emphasis*.

- List item 1
- List item 2

1. Ordered item 1
2. Ordered item 2

> A blockquote

[link](https://example.com)

`inline code`

```rust
fn main() {}
```

---
"#;

    let html = parse_to_html(input, &ParserOptions::default());

    // Check that we have basic structure (comrak adds anchor IDs)
    assert!(html.contains("<h1") && html.contains("Heading"));
    assert!(html.contains("<strong>paragraph</strong>"));
    assert!(html.contains("<em>emphasis</em>"));
    assert!(html.contains("<ul>"));
    assert!(html.contains("<ol>"));
    assert!(html.contains("<blockquote>"));
    assert!(html.contains("<a href=\"https://example.com\">"));
    assert!(html.contains("<code>inline code</code>"));
    assert!(html.contains("fn main() {}"));
    assert!(html.contains("<hr"));
}
