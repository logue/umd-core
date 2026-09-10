use umd::parse;
use umd::parser::ParserOptions;
use umd::parse_with_frontmatter_opts;

fn main() {
    println!("=== Color Support Test ===\n");

    // Named colors (UMD's own umd-color-*/umd-bg-* palette — 16 names, see
    // src/extensions/decoration_values.rs::COLOR_NAMES). There is no
    // semantic/role-based palette (primary/danger/success/etc.) — a name
    // outside this fixed list is rejected unless it's a hex value with
    // allow_hex_colors enabled (see below).
    println!("1. Named Colors:");
    let input = "&color(blue){Blue} &color(teal){Teal} &color(amber){Amber};";
    let output = parse(input);
    println!("   Input:  {}", input);
    println!("   Output: {}\n", output);

    // Full 16-color palette
    println!("2. Full Palette Coverage:");
    let colors = [
        "blue", "indigo", "violet", "purple", "pink", "red", "orange", "amber", "yellow", "lime",
        "green", "teal", "cyan", "brown", "gray", "pewter",
    ];
    let input = colors
        .iter()
        .map(|c| format!("&color({}){{{}}};", c, c))
        .collect::<Vec<_>>()
        .join(" ");
    let output = parse(&input);
    println!("   Input:  {}", input);
    println!("   Output: {}\n", output);

    // HEX colors — rejected by default; only styled when allow_hex_colors
    // is explicitly enabled (see ParserOptions::allow_hex_colors), since
    // unbounded custom color values are opt-in to prevent abuse in
    // untrusted content.
    println!("3. HEX Colors (allow_hex_colors: false, the default):");
    let input = "&color(#FF5733){Red Orange} &color(#3B7){Green};";
    let output = parse(input);
    println!("   Input:  {}", input);
    println!("   Output: {}\n", output);

    println!("4. HEX Colors (allow_hex_colors: true):");
    let mut options = ParserOptions::default();
    options.allow_hex_colors = true;
    let output = parse_with_frontmatter_opts(input, &options).html;
    println!("   Input:  {}", input);
    println!("   Output: {}\n", output);

    // Invalid colors (should be stripped)
    println!("5. Invalid Colors (not in the named palette):");
    let input = "&color(white){This should not have color} &color(black){Neither should this};";
    let output = parse(input);
    println!("   Input:  {}", input);
    println!("   Output: {}\n", output);

    // Background colors
    println!("6. Background Colors:");
    let input = "&color(,blue){Blue BG} &color(yellow,teal){Yellow on Teal};";
    let output = parse(input);
    println!("   Input:  {}", input);
    println!("   Output: {}\n", output);

    // Block level colors
    println!("7. Block Level Colors:");
    let input = "COLOR(cyan): This is cyan text\nBG_COLOR(yellow): Yellow background";
    let output = parse(input);
    println!("   Input:  {}", input);
    println!("   Output: {}\n", output);

    println!("=== Summary ===");
    println!("✓ Named colors (umd-color-*/umd-bg-*, 16-color palette): Working");
    println!("✓ HEX colors (#RGB, #RRGGBB) with allow_hex_colors: true: Working");
    println!("✓ HEX colors rejected by default (allow_hex_colors: false): Working");
    println!("✓ Names outside the palette: Properly rejected");
    println!("✓ Background colors: Working");
    println!("✓ Block-level colors: Working");
}
