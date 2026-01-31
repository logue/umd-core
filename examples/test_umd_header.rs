use universal_markdown::parse;

fn main() {
    // Test 1: Table without 'h' suffix - no <thead>
    println!("=== Test 1: Table without 'h' suffix ===");
    let input1 = r#"
| Cell A | Cell B |
| Cell C | Cell D |
"#;
    let output1 = parse(input1);
    println!("Input:\n{}", input1);
    println!("Output:\n{}\n", output1);

    // Test 2: Table with 'h' suffix - has <thead>
    println!("=== Test 2: Table with 'h' suffix ===");
    let input2 = r#"
| ~Header A | ~Header B |h
| Cell C | Cell D |
"#;
    let output2 = parse(input2);
    println!("Input:\n{}", input2);
    println!("Output:\n{}\n", output2);

    // Test 3: Mixed th/td in body rows
    println!("=== Test 3: Mixed th/td with row headers ===");
    let input3 = r#"
| ~Header A | ~Header B |h
| ~Row1 | Data 1 |
| ~Row2 | Data 2 |
"#;
    let output3 = parse(input3);
    println!("Input:\n{}", input3);
    println!("Output:\n{}\n", output3);

    // Test 4: Header marker after decoration
    println!("=== Test 4: Header marker with decorations ===");
    let input4 = r#"
| COLOR(blue): ~Header A | SIZE(1.5): ~Header B |h
| CENTER: ~Row1 | Data 1 |
"#;
    let output4 = parse(input4);
    println!("Input:\n{}", input4);
    println!("Output:\n{}\n", output4);

    // Test 5: Table without header markers but with 'h' suffix
    println!("=== Test 5: No ~ markers but has 'h' suffix ===");
    let input5 = r#"
| Header A | Header B |h
| Cell C | Cell D |
"#;
    let output5 = parse(input5);
    println!("Input:\n{}", input5);
    println!("Output:\n{}\n", output5);
}
