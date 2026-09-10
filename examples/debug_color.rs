//! Debug helper: reports which color names `&color()` currently accepts.
//!
//! This calls the real `umd::parse` and inspects its output rather than
//! re-implementing the matching logic locally (the real logic in
//! `src/extensions/decoration_values.rs::map_color_value_with_options` is
//! `pub(crate)`, not part of the public API, so an example binary can't
//! call it directly — and keeping an independent copy here is exactly the
//! kind of drift that led this file to test against a color list that no
//! longer matched the parser).
use umd::parse;

fn main() {
    // UMD's current named palette (src/extensions/decoration_values.rs::COLOR_NAMES).
    let known_colors = [
        "blue", "indigo", "violet", "purple", "pink", "red", "orange", "amber", "yellow", "lime",
        "green", "teal", "cyan", "brown", "gray", "pewter",
    ];

    // A few names that are deliberately NOT in the palette, to show they're
    // rejected: "white"/"black" (never supported), and the old
    // semantic/role-based names ("primary" etc.) that UMD dropped in favor
    // of letting the host application decide what "primary" maps to.
    let test_colors = [
        "blue", "red", "yellow", "pewter", "primary", "danger", "white",
    ];

    for test_color in test_colors {
        let input = format!("&color({}){{text}};", test_color);
        let output = parse(&input);
        let expected_class = format!("class=\"umd-color-{}\"", test_color);

        if output.contains(&expected_class) {
            println!(
                "✓ '{}' accepted - class=\"umd-color-{}\"",
                test_color, test_color
            );
        } else {
            println!("✗ '{}' NOT accepted - color stripped", test_color);
        }
    }

    println!(
        "\nFull palette ({} colors): {}",
        known_colors.len(),
        known_colors.join(", ")
    );
}
