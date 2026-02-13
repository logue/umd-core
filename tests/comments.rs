//! Tests for comment syntax

use umd::parse_with_frontmatter;

#[test]
fn test_single_line_comment_whole_line() {
    let input = "// この行はコメント\n通常のテキスト";
    let result = parse_with_frontmatter(input);
    
    assert!(!result.html.contains("この行はコメント"));
    assert!(result.html.contains("通常のテキスト"));
}

#[test]
fn test_single_line_comment_inline() {
    let input = "表示される // 表示されない";
    let result = parse_with_frontmatter(input);
    
    assert!(result.html.contains("表示される"));
    assert!(!result.html.contains("表示されない"));
}

#[test]
fn test_multiline_comment_block() {
    let input = "テキスト1\n/* コメント開始\n複数行\nコメント終了 */\nテキスト2";
    let result = parse_with_frontmatter(input);
    
    assert!(result.html.contains("テキスト1"));
    assert!(result.html.contains("テキスト2"));
    assert!(!result.html.contains("コメント開始"));
    assert!(!result.html.contains("複数行"));
    assert!(!result.html.contains("コメント終了"));
}

#[test]
fn test_multiline_comment_inline() {
    let input = "前部分/* コメント */後部分";
    let result = parse_with_frontmatter(input);
    
    assert!(result.html.contains("前部分"));
    assert!(result.html.contains("後部分"));
    assert!(!result.html.contains("コメント"));
}

#[test]
fn test_comment_in_code_block() {
    let input = "```rust\n// Rustのコメント\nfn main() {}\n```";
    let result = parse_with_frontmatter(input);
    
    // コードブロック内のコメントは保持される
    assert!(result.html.contains("// Rustのコメント"));
}

#[test]
fn test_comment_in_inline_code() {
    let input = "通常テキスト `// コード内コメント` 通常テキスト";
    let result = parse_with_frontmatter(input);
    
    // インラインコード内のコメントは保持される
    assert!(result.html.contains("// コード内コメント"));
}

#[test]
fn test_multiple_single_line_comments() {
    let input = "// コメント1\nテキスト1\n// コメント2\nテキスト2";
    let result = parse_with_frontmatter(input);
    
    assert!(result.html.contains("テキスト1"));
    assert!(result.html.contains("テキスト2"));
    assert!(!result.html.contains("コメント1"));
    assert!(!result.html.contains("コメント2"));
}

#[test]
fn test_nested_multiline_comments_not_supported() {
    // ネストはサポートしない（C言語スタイル）
    let input = "/* 外側 /* 内側 */ 続き */";
    let result = parse_with_frontmatter(input);
    
    // 最初の */ で閉じられるため、"続き */" が残る
    assert!(result.html.contains("続き */"));
    assert!(!result.html.contains("外側"));
    assert!(!result.html.contains("内側"));
}

#[test]
fn test_comment_with_umd_syntax() {
    let input = "// COLOR:red テキスト\nCOLOR:blue 表示されるテキスト";
    let result = parse_with_frontmatter(input);
    
    assert!(!result.html.contains("COLOR:red"));
    assert!(result.html.contains("表示されるテキスト"));
}

#[test]
fn test_comment_preserves_markdown() {
    let input = "# ヘッダー\n// コメント\n**太字**";
    let result = parse_with_frontmatter(input);
    
    assert!(result.html.contains("<h1"));
    assert!(result.html.contains("ヘッダー"));
    assert!(result.html.contains("<strong>"));
    assert!(result.html.contains("太字"));
    assert!(!result.html.contains("コメント"));
}

#[test]
fn test_empty_lines_after_comment_removal() {
    let input = "テキスト1\n// コメント行\n\nテキスト2";
    let result = parse_with_frontmatter(input);
    
    assert!(result.html.contains("テキスト1"));
    assert!(result.html.contains("テキスト2"));
}

#[test]
fn test_comment_at_end_of_line_with_period() {
    let input = "文章です。// コメント";
    let result = parse_with_frontmatter(input);
    
    assert!(result.html.contains("文章です。"));
    assert!(!result.html.contains("コメント"));
}

#[test]
fn test_multiline_comment_across_paragraphs() {
    let input = "段落1\n\n/* コメント\n\n段落2も含む */\n\n段落3";
    let result = parse_with_frontmatter(input);
    
    assert!(result.html.contains("段落1"));
    assert!(result.html.contains("段落3"));
    assert!(!result.html.contains("段落2"));
}

#[test]
fn test_url_with_double_slash_not_comment() {
    let input = "リンク: https://example.com/path";
    let result = parse_with_frontmatter(input);
    
    // URLの//はコメントではない（hの直後だから）
    assert!(result.html.contains("https://example.com"));
}
