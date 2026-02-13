use umd::parse_with_frontmatter;

fn main() {
    let tests = vec![
        ("// コメント\nテキスト", "single line"),
        ("前// コメント", "inline single"),
        ("前/* コメント */後", "inline multi"),
        ("テキスト1\n/* コメント */\nテキスト2", "multi block"),
        ("https://example.com/path", "URL"),
        ("リンク: https://example.com/path", "URL with text"),
    ];

    for (input, desc) in tests {
        println!("\n=== {} ===", desc);
        println!("Input:\n{}", input);
        let result = parse_with_frontmatter(input);
        println!("Output:\n{}", result.html);
    }
}
