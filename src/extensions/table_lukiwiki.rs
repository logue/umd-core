//! LukiWiki table parser
//!
//! Provides support for LukiWiki-style tables with extended features:
//! - Cell spanning: `|>` for colspan, `|^` for rowspan
//! - Cell decorations: COLOR(), SIZE(), alignment prefixes
//! - No mandatory header row (unlike GFM)

use regex::Regex;

/// Check if a table is LukiWiki format or GFM format
///
/// LukiWiki format is identified by:
/// - 2nd line is NOT a GFM separator line (only `|`, `:`, `-`)
/// - OR contains LukiWiki-specific markers (`|>`, `|^`, decoration prefixes)
/// - OR only 1 line (GFM requires at least 2 lines)
///
/// # Arguments
///
/// * `lines` - Table lines
///
/// # Returns
///
/// true if LukiWiki format, false if GFM format
fn is_lukiwiki_table(lines: &[&str]) -> bool {
    if lines.is_empty() {
        return false;
    }

    // Single-line tables are LukiWiki format (GFM requires header + separator)
    if lines.len() == 1 {
        return true;
    }

    // Check if 2nd line is a GFM separator
    let second_line = lines[1].trim();
    let is_gfm_separator = second_line
        .chars()
        .all(|c| c == '|' || c == ':' || c == '-' || c.is_whitespace());

    if !is_gfm_separator {
        // If 2nd line is not GFM separator, it's LukiWiki format
        return true;
    }

    // Check for LukiWiki-specific markers in any line
    for line in lines {
        if line.contains("|>")
            || line.contains("|^")
            || line.contains("COLOR(")
            || line.contains("SIZE(")
            || line.contains("TOP:")
            || line.contains("MIDDLE:")
            || line.contains("BOTTOM:")
            || line.contains("CENTER:")
            || line.contains("RIGHT:")
            || line.contains("LEFT:")
        {
            return true;
        }
    }

    // Otherwise, it's GFM format
    false
}

/// Cell information
#[derive(Debug, Clone)]
struct Cell {
    content: String,
    is_header: bool,
    colspan: usize,
    rowspan: usize,
    classes: Vec<String>,
    styles: Vec<String>,
}

impl Cell {
    fn new(content: String, is_header: bool) -> Self {
        Self {
            content,
            is_header,
            colspan: 1,
            rowspan: 1,
            classes: Vec::new(),
            styles: Vec::new(),
        }
    }
}

/// Parse a LukiWiki table and convert to HTML
///
/// # Arguments
///
/// * `table_text` - The table text (multiple lines starting with |)
///
/// # Returns
///
/// HTML table string
pub fn parse_lukiwiki_table(table_text: &str) -> String {
    let lines: Vec<&str> = table_text.lines().collect();

    if lines.is_empty() {
        return String::new();
    }

    // Check if this is actually a LukiWiki table
    if !is_lukiwiki_table(&lines) {
        // Return as-is, let comrak handle it
        return table_text.to_string();
    }

    // Parse table rows
    let mut rows: Vec<Vec<Cell>> = Vec::new();

    for line in &lines {
        let line = line.trim();
        if line.is_empty() || !line.starts_with('|') {
            continue;
        }

        // Parse cells more carefully to handle |> and |^ markers
        let mut cells: Vec<Cell> = Vec::new();
        let mut current_pos = 1; // Skip first |
        let chars: Vec<char> = line.chars().collect();
        let mut current_cell = String::new();

        while current_pos < chars.len() {
            if chars[current_pos] == '|' {
                // Check if this is |> or |^ marker
                if current_pos + 1 < chars.len() {
                    let next_char = chars[current_pos + 1];
                    if next_char == '>' || next_char == '^' {
                        // This is a marker, include it in the cell
                        current_cell.push('|');
                        current_cell.push(next_char);
                        current_pos += 2;
                        continue;
                    }
                }

                // Regular cell separator
                let content = current_cell.trim().to_string();
                let is_header = rows.is_empty();
                let mut cell = Cell::new(content, is_header);
                parse_cell_content(&mut cell);
                cells.push(cell);
                current_cell.clear();
                current_pos += 1;
            } else {
                current_cell.push(chars[current_pos]);
                current_pos += 1;
            }
        }

        // Add last cell if exists
        if !current_cell.trim().is_empty() || !cells.is_empty() {
            let content = current_cell.trim().to_string();
            let is_header = rows.is_empty();
            let mut cell = Cell::new(content, is_header);
            parse_cell_content(&mut cell);
            cells.push(cell);
        }

        // Don't remove trailing empty cells - they may be part of colspan

        rows.push(cells);
    }

    // Process cell spanning
    process_cell_spanning(&mut rows);

    // Generate HTML
    generate_table_html(&rows)
}

/// Parse cell content for decorations and markers
fn parse_cell_content(cell: &mut Cell) {
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

    // Parse COLOR(fg,bg):
    let color_pattern = Regex::new(r"^COLOR\(([^)]*)\):\s*(.*)$").unwrap();
    if let Some(caps) = color_pattern.captures(&remaining) {
        let args = caps[1].to_string();
        remaining = caps[2].to_string();

        let parts: Vec<&str> = args.split(',').collect();
        let fg = parts.get(0).map_or("", |s| s.trim());
        let bg = parts.get(1).map_or("", |s| s.trim());

        if !fg.is_empty() && fg != "inherit" {
            if is_bootstrap_color(fg) {
                cell.classes.push(format!("text-{}", fg));
            } else {
                cell.styles.push(format!("color: {}", fg));
            }
        }

        if !bg.is_empty() && bg != "inherit" {
            if is_bootstrap_color(bg) {
                cell.classes.push(format!("bg-{}", bg));
            } else {
                cell.styles.push(format!("background-color: {}", bg));
            }
        }
    }

    // Parse SIZE(value):
    let size_pattern = Regex::new(r"^SIZE\(([^)]+)\):\s*(.*)$").unwrap();
    if let Some(caps) = size_pattern.captures(&remaining) {
        let value = caps[1].to_string();
        remaining = caps[2].to_string();

        // Check if it's a Bootstrap size
        if let Some(bs_class) = get_bootstrap_size_class(&value) {
            cell.classes.push(bs_class);
        } else {
            // Use inline style
            let size_value =
                if value.contains("rem") || value.contains("em") || value.contains("px") {
                    value.to_string()
                } else {
                    format!("{}rem", value)
                };
            cell.styles.push(format!("font-size: {}", size_value));
        }
    }

    // Parse alignment prefixes
    for (prefix, class) in &[
        ("TOP:", "align-top"),
        ("MIDDLE:", "align-middle"),
        ("BOTTOM:", "align-bottom"),
        ("BASELINE:", "align-baseline"),
        ("RIGHT:", "text-end"),
        ("CENTER:", "text-center"),
        ("LEFT:", "text-start"),
        ("JUSTIFY:", "text-justify"),
    ] {
        if remaining.starts_with(prefix) {
            cell.classes.push(class.to_string());
            remaining = remaining.strip_prefix(prefix).unwrap().trim().to_string();
        }
    }

    cell.content = remaining;
}

/// Check if a color is a Bootstrap color name
fn is_bootstrap_color(color: &str) -> bool {
    matches!(
        color,
        "primary"
            | "secondary"
            | "success"
            | "danger"
            | "warning"
            | "info"
            | "light"
            | "dark"
            | "blue"
            | "indigo"
            | "purple"
            | "pink"
            | "red"
            | "orange"
            | "yellow"
            | "green"
            | "teal"
            | "cyan"
            | "black"
            | "white"
            | "gray"
            | "gray-dark"
            | "gray-100"
            | "gray-200"
            | "gray-300"
            | "gray-400"
            | "gray-500"
            | "gray-600"
            | "gray-700"
            | "gray-800"
            | "gray-900"
    )
}

/// Get Bootstrap size class for a given value
fn get_bootstrap_size_class(value: &str) -> Option<String> {
    let val: f32 = value.parse().ok()?;

    let class = if val >= 2.5 {
        "fs-1"
    } else if val >= 2.0 {
        "fs-2"
    } else if val >= 1.75 {
        "fs-3"
    } else if val >= 1.5 {
        "fs-4"
    } else if val >= 1.25 {
        "fs-5"
    } else if val >= 0.875 {
        "fs-6"
    } else {
        return None;
    };

    Some(class.to_string())
}

/// Process cell spanning (colspan and rowspan)
fn process_cell_spanning(rows: &mut Vec<Vec<Cell>>) {
    // Process colspan
    for row in rows.iter_mut() {
        let mut i = 0;
        while i < row.len() {
            if row[i].content.ends_with("|>") || row[i].content == "|>" {
                // Remove marker
                row[i].content = row[i].content.trim_end_matches("|>").trim().to_string();

                // Count consecutive empty or |> cells
                let mut span = 1;
                let j = i + 1;
                while j < row.len() && (row[j].content.is_empty() || row[j].content == "|>") {
                    span += 1;
                    row.remove(j); // Remove merged cell
                }
                row[i].colspan = span;
            }
            i += 1;
        }
    }

    // Process rowspan
    let max_cols = rows.iter().map(|r| r.len()).max().unwrap_or(0);
    for col in 0..max_cols {
        let mut row_idx = 0;
        while row_idx < rows.len() {
            if col < rows[row_idx].len() && rows[row_idx][col].content == "|^" {
                // Find source cell above
                if row_idx > 0 && col < rows[row_idx - 1].len() {
                    rows[row_idx - 1][col].rowspan += 1;
                    rows[row_idx].remove(col);
                }
            }
            row_idx += 1;
        }
    }
}

/// Generate HTML table from parsed cells
fn generate_table_html(rows: &[Vec<Cell>]) -> String {
    // Add data-lukiwiki attribute to distinguish from GFM tables
    let mut html = String::from(r#"<table class="table" data-lukiwiki="true">"#);

    if rows.is_empty() {
        html.push_str("</table>");
        return html;
    }

    // Determine if first row is header row
    let has_header = rows
        .first()
        .map_or(false, |r| r.iter().any(|c| c.is_header));

    if has_header {
        // Generate <thead> for first row
        html.push_str("<thead>");
        if let Some(header_row) = rows.first() {
            html.push_str("<tr>");
            for cell in header_row {
                let tag = "th";

                // Build attributes
                let mut attrs = Vec::new();

                if !cell.classes.is_empty() {
                    attrs.push(format!(r#"class="{}""#, cell.classes.join(" ")));
                }

                if !cell.styles.is_empty() {
                    attrs.push(format!(r#"style="{}""#, cell.styles.join("; ")));
                }

                if cell.colspan > 1 {
                    attrs.push(format!(r#"colspan="{}""#, cell.colspan));
                }

                if cell.rowspan > 1 {
                    attrs.push(format!(r#"rowspan="{}""#, cell.rowspan));
                }

                let attrs_str = if attrs.is_empty() {
                    String::new()
                } else {
                    format!(" {}", attrs.join(" "))
                };

                html.push_str(&format!("<{tag}{attrs_str}>{}</{tag}>", cell.content));
            }
            html.push_str("</tr>");
        }
        html.push_str("</thead>");
    }

    // Generate <tbody> for remaining rows
    let body_rows = if has_header { &rows[1..] } else { rows };

    if !body_rows.is_empty() {
        html.push_str("<tbody>");
        for row in body_rows {
            html.push_str("<tr>");
            for cell in row {
                let tag = if cell.is_header { "th" } else { "td" };

                // Build attributes
                let mut attrs = Vec::new();

                if !cell.classes.is_empty() {
                    attrs.push(format!(r#"class="{}""#, cell.classes.join(" ")));
                }

                if !cell.styles.is_empty() {
                    attrs.push(format!(r#"style="{}""#, cell.styles.join("; ")));
                }

                if cell.colspan > 1 {
                    attrs.push(format!(r#"colspan="{}""#, cell.colspan));
                }

                if cell.rowspan > 1 {
                    attrs.push(format!(r#"rowspan="{}""#, cell.rowspan));
                }

                let attrs_str = if attrs.is_empty() {
                    String::new()
                } else {
                    format!(" {}", attrs.join(" "))
                };

                html.push_str(&format!("<{tag}{attrs_str}>{}</{tag}>", cell.content));
            }
            html.push_str("</tr>");
        }
        html.push_str("</tbody>");
    }

    html.push_str("</table>");
    html
}

/// Detect and extract LukiWiki tables from input text
///
/// Returns a tuple of (processed_text, table_map)
/// where table_map contains markers and their corresponding HTML
pub fn extract_lukiwiki_tables(input: &str) -> (String, Vec<(String, String)>) {
    let mut result = input.to_string();
    let mut tables = Vec::new();
    let mut table_counter = 0;

    // Find all potential tables (consecutive lines starting with |)
    let mut in_table = false;
    let mut table_lines = Vec::new();
    let lines: Vec<&str> = input.lines().collect();

    for line in lines.iter() {
        let trimmed = line.trim();

        if trimmed.starts_with('|') {
            if !in_table {
                in_table = true;
                table_lines.clear();
            }
            table_lines.push(*line);
        } else {
            if in_table && !table_lines.is_empty() {
                // End of table
                let table_text = table_lines.join("\n");

                // Check if it's LukiWiki format
                let table_lines_refs: Vec<&str> = table_text.lines().collect();
                if is_lukiwiki_table(&table_lines_refs) {
                    // Parse and replace with marker
                    let html = parse_lukiwiki_table(&table_text);
                    // Use a marker with newlines to make comrak treat it as block-level
                    let marker = format!("\n\nLUKIWIKI_TABLE_MARKER_{}_END\n\n", table_counter);
                    tables.push((marker.clone(), html));

                    // Replace in result
                    result = result.replace(&table_text, &marker);
                    table_counter += 1;
                }

                table_lines.clear();
                in_table = false;
            }
        }
    }

    // Handle table at end of file
    if in_table && !table_lines.is_empty() {
        let table_text = table_lines.join("\n");
        let table_lines_refs: Vec<&str> = table_text.lines().collect();
        if is_lukiwiki_table(&table_lines_refs) {
            let html = parse_lukiwiki_table(&table_text);
            // Use a marker with newlines
            let marker = format!("\n\nLUKIWIKI_TABLE_MARKER_{}_END\n\n", table_counter);
            tables.push((marker.clone(), html));
            result = result.replace(&table_text, &marker);
        }
    }

    (result, tables)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_lukiwiki_table() {
        let lukiwiki = vec!["| A | B |", "| C | D |"];
        assert!(is_lukiwiki_table(&lukiwiki));

        let gfm = vec!["| A | B |", "|---|---|", "| C | D |"];
        assert!(!is_lukiwiki_table(&gfm));

        let with_colspan = vec!["| A |> | C |", "|---|---|---|", "| D | E | F |"];
        assert!(is_lukiwiki_table(&with_colspan));
    }

    #[test]
    fn test_parse_simple_table() {
        let input = "| A | B |\n| C | D |";
        let html = parse_lukiwiki_table(input);
        assert!(html.contains(r#"<table class="table" data-lukiwiki="true">"#));
        assert!(html.contains("<thead>"));
        assert!(html.contains("<tbody>"));
        assert!(html.contains("<th>A</th>"));
        assert!(html.contains("<td>C</td>"));
    }

    #[test]
    fn test_parse_colspan() {
        let input = "| A |> |\n| C | D |";
        let html = parse_lukiwiki_table(input);
        eprintln!("Input: {}", input);
        eprintln!("Output: {}", html);
        assert!(html.contains("<thead>"));
        assert!(html.contains("<tbody>"));
        assert!(html.contains(r#"colspan="2""#));
    }

    #[test]
    fn test_parse_with_decoration() {
        let input = "| COLOR(red): A | B |";
        let html = parse_lukiwiki_table(input);
        eprintln!("Input: {}", input);
        eprintln!("Output: {}", html);
        // Bootstrap color names are output as classes
        assert!(html.contains("class="));
        assert!(html.contains("text-red"));
    }
}
