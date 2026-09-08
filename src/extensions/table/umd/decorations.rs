//! Cell decoration support for UMD tables
//!
//! Provides support for:
//! - COLOR(fg,bg): Cell foreground and background colors
//! - SIZE(value): Font size adjustments
//! - Alignment prefixes: V-START:, V-CENTER:, V-END:, CENTER:, etc.
//!   (logical-direction names, matching block_decorations.rs and
//!   scss/utilities/text.scss — not physical TOP:/BOTTOM:/LEFT:/RIGHT:)
//!
//! `COLOR()`/`SIZE()` share the same value palette and class naming as
//! `&color()`/`&size()` (see `decoration_values.rs`) — the applied-to
//! element is the only difference: those inline plugins wrap a `<span>`
//! within the text, while here the class/style is attached to the cell
//! itself (`<td>`/`<th>`), since cell background/foreground color is a
//! property of the cell, not of a run of text. A nested `&color()` inside
//! cell content still works independently to override part of the text.

use super::parser::Cell;
use crate::extensions::decoration_values::{map_color_value_with_options, map_font_size_value};
use regex::Regex;

/// Parse cell content for decorations and markers
pub fn parse_cell_content(cell: &mut Cell, allow_hex_colors: bool, allow_custom_font_size: bool) {
    let content = &cell.content;

    // Check for colspan marker: |> - mark but don't remove yet
    if content == "|>" || content.ends_with(" |>") {
        // Keep the marker for process_cell_spanning
        return;
    }

    // Check for rowspan marker: |^
    if content == "|^" {
        // Keep the marker for process_cell_spanning
        return;
    }

    // Parse decoration prefixes
    let mut remaining = content.clone();

    // Check for header marker: ~
    if remaining.starts_with('~') {
        cell.is_header = true;
        remaining = remaining.strip_prefix('~').unwrap().trim().to_string();
    }

    // Parse COLOR(fg,bg):
    let color_pattern = Regex::new(r"^COLOR\(([^)]*)\):\s*(.*)$").unwrap();
    if let Some(caps) = color_pattern.captures(&remaining) {
        let args = caps[1].to_string();
        remaining = caps[2].to_string();

        let parts: Vec<&str> = args.split(',').collect();
        let fg = parts.get(0).map_or("", |s| s.trim());
        let bg = parts.get(1).map_or("", |s| s.trim());

        if !fg.is_empty() && fg != "inherit" {
            if let Some((is_class, value)) =
                map_color_value_with_options(fg, false, allow_hex_colors)
            {
                if is_class {
                    cell.classes.push(value);
                } else {
                    cell.styles.push(format!("color: {}", value));
                }
            }
        }

        if !bg.is_empty() && bg != "inherit" {
            if let Some((is_class, value)) =
                map_color_value_with_options(bg, true, allow_hex_colors)
            {
                if is_class {
                    cell.classes.push(value);
                } else {
                    cell.styles.push(format!("background-color: {}", value));
                }
            }
        }
    }

    // Parse SIZE(value):
    let size_pattern = Regex::new(r"^SIZE\(([^)]+)\):\s*(.*)$").unwrap();
    if let Some(caps) = size_pattern.captures(&remaining) {
        let value = caps[1].to_string();
        remaining = caps[2].to_string();

        match map_font_size_value(&value, allow_custom_font_size) {
            Some((true, class)) => cell.classes.push(class),
            Some((false, style_value)) => {
                cell.styles.push(format!("font-size: {}", style_value))
            }
            None => {}
        }
    }

    // Parse alignment prefixes (keyword/class table shared with the rest of
    // the alignment scope — see src/extensions/alignment.rs)
    for (keyword, class) in crate::extensions::alignment::cell_alignment_prefixes() {
        if let Some(stripped) = remaining
            .strip_prefix(keyword)
            .and_then(|rest| rest.strip_prefix(':'))
        {
            cell.classes.push(class.to_string());
            remaining = stripped.trim().to_string();
        }
    }

    // Check for header marker after decoration prefixes: ~
    if remaining.starts_with('~') {
        cell.is_header = true;
        remaining = remaining.strip_prefix('~').unwrap().trim().to_string();
    }

    cell.content = remaining;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_decoration() {
        let mut cell = Cell::new("COLOR(red,blue): Text".to_string(), false);
        parse_cell_content(&mut cell, false, false);

        assert_eq!(cell.content, "Text");
        assert!(cell.classes.contains(&"umd-color-red".to_string()));
        assert!(cell.classes.contains(&"umd-bg-blue".to_string()));
        assert!(!cell.is_header);
    }

    #[test]
    fn test_color_decoration_hex_requires_opt_in() {
        let mut cell = Cell::new("COLOR(#ff0000): Text".to_string(), false);
        parse_cell_content(&mut cell, false, false);
        assert!(cell.classes.is_empty());
        assert!(cell.styles.is_empty());

        let mut cell = Cell::new("COLOR(#ff0000): Text".to_string(), false);
        parse_cell_content(&mut cell, true, false);
        assert!(cell.styles.contains(&"color: #ff0000".to_string()));
    }

    #[test]
    fn test_header_marker() {
        let mut cell = Cell::new("~Header Text".to_string(), false);
        parse_cell_content(&mut cell, false, false);

        assert_eq!(cell.content, "Header Text");
        assert!(cell.is_header);
    }

    #[test]
    fn test_header_marker_with_decoration() {
        let mut cell = Cell::new("COLOR(red): ~Header".to_string(), false);
        parse_cell_content(&mut cell, false, false);

        assert_eq!(cell.content, "Header");
        assert!(cell.is_header);
        assert!(cell.classes.contains(&"umd-color-red".to_string()));
    }

    #[test]
    fn test_size_decoration_keyword() {
        let mut cell = Cell::new("SIZE(lg): Text".to_string(), false);
        parse_cell_content(&mut cell, false, false);

        assert_eq!(cell.content, "Text");
        assert!(cell.classes.contains(&"umd-text-size-lg".to_string()));
    }

    #[test]
    fn test_size_decoration_custom_requires_opt_in() {
        let mut cell = Cell::new("SIZE(1.5): Text".to_string(), false);
        parse_cell_content(&mut cell, false, false);
        assert!(cell.classes.is_empty());
        assert!(cell.styles.is_empty());

        let mut cell = Cell::new("SIZE(1.5): Text".to_string(), false);
        parse_cell_content(&mut cell, false, true);
        assert!(cell.styles.contains(&"font-size: 1.5rem".to_string()));
    }

    #[test]
    fn test_alignment_decoration() {
        let mut cell = Cell::new("CENTER: Text".to_string(), false);
        parse_cell_content(&mut cell, false, false);

        assert_eq!(cell.content, "Text");
        assert!(cell.classes.contains(&"umd-center".to_string()));
    }

    #[test]
    fn test_vertical_alignment_decoration() {
        let mut cell = Cell::new("V-START: Text".to_string(), false);
        parse_cell_content(&mut cell, false, false);

        assert_eq!(cell.content, "Text");
        assert!(cell.classes.contains(&"umd-v-start".to_string()));
    }
}
