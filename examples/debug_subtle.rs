// Debug blue-subtle matching
fn main() {
    let trimmed = "blue-subtle";

    let bootstrap_colors = ["primary", "blue", "blue-subtle", "blue-emphasis"];

    for color in &bootstrap_colors {
        if trimmed == *color {
            println!("✓ Exact match: {} == {}", trimmed, color);
            let prefix = "text";
            println!("  Result: (true, \"{}-{}\")", prefix, trimmed);
            break;
        } else if trimmed.starts_with(&format!("{}-", color)) {
            println!("✓ Prefix match: {} starts_with {}-", trimmed, color);
            let prefix = "text";
            let result = format!("{}-{}", prefix, trimmed);
            println!("  Result: (true, \"{}\")", result);
            break;
        } else {
            println!("✗ No match: {} against {}", trimmed, color);
        }
    }
}
