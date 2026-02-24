// Debug color mapping
fn main() {
    // Bootstrap theme colors
    let bootstrap_colors = [
        // Theme colors
        "primary",
        "secondary",
        "success",
        "danger",
        "warning",
        "info",
        "light",
        "dark",
        "body",
        "body-secondary",
        "body-tertiary",
        "body-emphasis",
        // Custom colors (Bootstrap 5.3+)
        "blue",
        "indigo",
        "purple",
        "pink",
        "red",
        "orange",
        "yellow",
        "green",
        "teal",
        "cyan",
        // Theme colors with suffixes
        "primary-subtle",
        "secondary-subtle",
        "success-subtle",
        "danger-subtle",
        "warning-subtle",
        "info-subtle",
        "light-subtle",
        "dark-subtle",
        "primary-emphasis",
        "secondary-emphasis",
        "success-emphasis",
        "danger-emphasis",
        "warning-emphasis",
        "info-emphasis",
        "light-emphasis",
        "dark-emphasis",
        // Custom colors with suffixes
        "blue-subtle",
        "indigo-subtle",
        "purple-subtle",
        "pink-subtle",
        "red-subtle",
        "orange-subtle",
        "yellow-subtle",
        "green-subtle",
        "teal-subtle",
        "cyan-subtle",
        "blue-emphasis",
        "indigo-emphasis",
        "purple-emphasis",
        "pink-emphasis",
        "red-emphasis",
        "orange-emphasis",
        "yellow-emphasis",
        "green-emphasis",
        "teal-emphasis",
        "cyan-emphasis",
    ];

    let test_colors = vec!["blue", "red", "yellow", "primary", "blue-subtle"];

    for test_color in test_colors {
        let trimmed = test_color;
        let mut found = false;

        for color in &bootstrap_colors {
            if trimmed == *color || trimmed.starts_with(&format!("{}-", color)) {
                println!(
                    "✓ '{}' matched with '{}' - Would return (true, \"text-{}\")",
                    test_color, color, trimmed
                );
                found = true;
                break;
            }
        }

        if !found {
            println!(
                "✗ '{}' NOT matched - Would return (false, \"{}\")",
                test_color, test_color
            );
        }
    }
}
