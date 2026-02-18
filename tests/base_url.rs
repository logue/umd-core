use umd::{parse_with_frontmatter_opts, parser::ParserOptions};

#[test]
fn test_base_url_with_links() {
    let input = "[docs](/docs)\n[api](/api/v1)";
    let mut opts = ParserOptions::default();
    opts.base_url = Some("/app".to_string());

    let result = parse_with_frontmatter_opts(input, &opts);
    assert!(result.html.contains(r#"href="/app/docs""#));
    assert!(result.html.contains(r#"href="/app/api/v1""#));
}

#[test]
fn test_base_url_with_media() {
    let input = "![logo](/logo.png)\n![banner](/images/banner.jpg)";
    let mut opts = ParserOptions::default();
    opts.base_url = Some("/assets".to_string());

    let result = parse_with_frontmatter_opts(input, &opts);
    assert!(result.html.contains(r#"src="/assets/logo.png""#));
    assert!(result.html.contains(r#"src="/assets/images/banner.jpg""#));
}

#[test]
fn test_base_url_with_trailing_slash() {
    let input = "[home](/home)";
    let mut opts = ParserOptions::default();
    opts.base_url = Some("/app/".to_string());

    let result = parse_with_frontmatter_opts(input, &opts);
    // Trailing slash should be removed
    assert!(result.html.contains(r#"href="/app/home""#));
}

#[test]
fn test_base_url_with_full_url() {
    let input = "[docs](/docs)";
    let mut opts = ParserOptions::default();
    opts.base_url = Some("https://example.com/app".to_string());

    let result = parse_with_frontmatter_opts(input, &opts);
    assert!(
        result
            .html
            .contains(r#"href="https://example.com/app/docs""#)
    );
}

#[test]
fn test_base_url_preserves_external_urls() {
    let input = "[external](https://example.com/page)\n[external2](http://example.com)";
    let mut opts = ParserOptions::default();
    opts.base_url = Some("/app".to_string());

    let result = parse_with_frontmatter_opts(input, &opts);
    // External URLs should not be modified
    assert!(result.html.contains(r#"href="https://example.com/page""#));
    assert!(result.html.contains(r#"href="http://example.com""#));
}

#[test]
fn test_base_url_with_single_quotes() {
    let input = r#"<a href='/docs'>Docs</a>"#;
    let mut opts = ParserOptions::default();
    opts.base_url = Some("/app".to_string());

    let result = parse_with_frontmatter_opts(input, &opts);
    assert!(result.html.contains(r#"href='/app/docs'"#));
}

#[test]
fn test_base_url_none() {
    let input = "[docs](/docs)";
    let opts = ParserOptions::default();

    let result = parse_with_frontmatter_opts(input, &opts);
    // Without base_url, paths should remain as-is
    assert!(result.html.contains(r#"href="/docs""#));
}

#[test]
fn test_base_url_with_mixed_links() {
    let input = "[home](/)\n[docs](/docs)\n[external](https://example.com)";
    let mut opts = ParserOptions::default();
    opts.base_url = Some("/app".to_string());

    let result = parse_with_frontmatter_opts(input, &opts);
    assert!(result.html.contains(r#"href="/app/""#));
    assert!(result.html.contains(r#"href="/app/docs""#));
    assert!(result.html.contains(r#"href="https://example.com""#));
}
