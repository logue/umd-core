use umd::parse;

fn main() {
    println!("=== Badge Basic ===");
    let output = parse("Check this &badge(primary){New};");
    println!("{}", output);

    println!("\n=== Badge with Link ===");
    let output = parse("&badge(danger){[Error](/error)};");
    println!("{}", output);

    println!("\n=== Color Bootstrap ===");
    let output = parse("&color(primary){Primary text};");
    println!("{}", output);

    println!("\n=== Size Bootstrap ===");
    let output = parse("&size(1.5){Medium text};");
    println!("{}", output);

    println!("\n=== Size Custom ===");
    let output = parse("&size(3rem){Large text};");
    println!("{}", output);

    println!("\n=== Block Color ===");
    let output = parse("COLOR(success): This is a success message");
    println!("{}", output);

    println!("\n=== Blockquote Markdown ===");
    let output = parse("> This is a markdown quote");
    println!("{}", output);

    println!("\n=== Blockquote UMD ===");
    let output = parse("> This is a UMD quote <");
    println!("{}", output);

    println!("\n=== GFM Alert Note ===");
    let output = parse("> [!NOTE]\n> This is an informational note");
    println!("{}", output);

    println!("\n=== Compound Prefixes ===");
    let output = parse("SIZE(1.5): COLOR(primary): CENTER: Styled text");
    println!("{}", output);

    println!("\n=== Custom Color Blue ===");
    let output = parse("&color(blue){Blue text};");
    println!("{}", output);

    println!("\n=== Custom Color Yellow ===");
    let output = parse("&color(yellow){Yellow text};");
    println!("{}", output);

    println!("\n=== Block Color Yellow ===");
    let output = parse("COLOR(yellow): Yellow block");
    println!("{}", output);

    println!("\n=== Custom Color Blue-Subtle ===");
    let output = parse("&color(blue-subtle){Blue subtle text};");
    println!("{}", output);

    println!("\n=== Background Blue ===");
    let output = parse("&color(,blue){Text on blue};");
    println!("{}", output);
}
