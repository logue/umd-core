use umd::parse_with_frontmatter;

fn main() {
    let tests = vec![
        (
            "Text before\n\n----\n\nText after\n\n***\n\nMore text",
            "HR",
        ),
        ("COLOR(blue): Blue paragraph", "COLOR"),
    ];

    for (input, desc) in tests {
        println!("\n=== {} ===", desc);
        println!("Input:\n{}", input);
        let result = parse_with_frontmatter(input);
        println!("Output:\n{}", result.html);
    }
}
