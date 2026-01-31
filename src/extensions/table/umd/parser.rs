//! UMD table parser
//!
//! Parses UMD-style table syntax into structured cell data

/// Cell information
#[derive(Debug, Clone)]
pub struct Cell {
    pub content: String,
    pub is_header: bool,
    pub colspan: usize,
    pub rowspan: usize,
    pub classes: Vec<String>,
    pub styles: Vec<String>,
}

impl Cell {
    pub fn new(content: String, is_header: bool) -> Self {
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

/// Check if a table is UMD format or GFM format
///
/// UMD format is identified by:
/// - 2nd line is NOT a GFM separator line (only `|`, `:`, `-`)
/// - OR contains UMD-specific markers (`|>`, `|^`, decoration prefixes)
/// - OR only 1 line (GFM requires at least 2 lines)
///
/// # Arguments
///
/// * `lines` - Table lines
///
/// # Returns
///
/// true if UMD format, false if GFM format
pub fn is_umd_table(lines: &[&str]) -> bool {
    if lines.is_empty() {
        return false;
    }

    // Single-line tables are UMD format (GFM requires header + separator)
    if lines.len() == 1 {
        return true;
    }

    // Check if 2nd line is a GFM separator
    let second_line = lines[1].trim();
    let is_gfm_separator = second_line
        .chars()
        .all(|c| c == '|' || c == ':' || c == '-' || c.is_whitespace());

    if !is_gfm_separator {
        // If 2nd line is not GFM separator, it's UMD format
        return true;
    }

    // Check for UMD-specific markers in any line
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

/// Parse a UMD table and convert to HTML
///
/// # Arguments
///
/// * `table_text` - The table text (multiple lines starting with |)
///
/// # Returns
///
/// HTML table string
pub fn parse_table(table_text: &str) -> String {
    let lines: Vec<&str> = table_text.lines().collect();

    if lines.is_empty() {
        return String::new();
    }

    // Check if this is actually a UMD table
    if !is_umd_table(&lines) {
        // Return as-is, let comrak handle it
        return table_text.to_string();
    }

    // Check if first row has 'h' suffix to determine if it's a header row
    let has_thead = lines
        .first()
        .map_or(false, |line| line.trim().ends_with("h"));

    // Parse table rows
    let mut rows: Vec<Vec<Cell>> = Vec::new();

    for (row_idx, line) in lines.iter().enumerate() {
        let mut line = line.trim();
        if line.is_empty() || !line.starts_with('|') {
            continue;
        }

        // Remove 'h' suffix from first row if present
        if row_idx == 0 && line.ends_with('h') {
            line = &line[..line.len() - 1];
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
                let mut cell = Cell::new(content, false);
                super::decorations::parse_cell_content(&mut cell);
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
            let mut cell = Cell::new(content, false);
            super::decorations::parse_cell_content(&mut cell);
            cells.push(cell);
        }

        // Don't remove trailing empty cells - they may be part of colspan

        rows.push(cells);
    }

    // Process cell spanning
    super::cell_spanning::process_cell_spanning(&mut rows);

    // Generate HTML with header information
    generate_table_html_with_header(&rows, has_thead)
}

/// Generate HTML table from parsed cells with header information
fn generate_table_html_with_header(rows: &[Vec<Cell>], has_thead: bool) -> String {
    // Add umd-table class to identify Universal Markdown tables
    let mut html = String::from(r#"<table class="table umd-table">"#);

    if rows.is_empty() {
        html.push_str("</table>");
        return html;
    }

    // Generate <thead> if first row has 'h' suffix
    if has_thead && !rows.is_empty() {
        html.push_str("<thead>");
        if let Some(header_row) = rows.first() {
            html.push_str("<tr>");
            for cell in header_row {
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
        html.push_str("</thead>");
    }

    // Generate <tbody> for remaining rows (or all rows if no thead)
    let body_rows = if has_thead { &rows[1..] } else { rows };

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

/// Detect and extract UMD tables from input text
///
/// Returns a tuple of (processed_text, table_map)
/// where table_map contains markers and their corresponding HTML
pub fn extract_umd_tables(input: &str) -> (String, Vec<(String, String)>) {
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

                // Check if it's UMD format
                let table_lines_refs: Vec<&str> = table_text.lines().collect();
                if is_umd_table(&table_lines_refs) {
                    // Parse and replace with marker
                    let html = parse_table(&table_text);
                    // Use a marker with newlines to make comrak treat it as block-level
                    let marker = format!("\n\nUMD_TABLE_MARKER_{}_END\n\n", table_counter);
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
        if is_umd_table(&table_lines_refs) {
            let html = parse_table(&table_text);
            // Use a marker with newlines
            let marker = format!("\n\nUMD_TABLE_MARKER_{}_END\n\n", table_counter);
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
    fn test_detect_umd_table() {
        let umd = vec!["| A | B |", "| C | D |"];
        assert!(is_umd_table(&umd));

        let gfm = vec!["| A | B |", "|---|---|", "| C | D |"];
        assert!(!is_umd_table(&gfm));

        let with_colspan = vec!["| A |> | C |", "|---|---|---|", "| D | E | F |"];
        assert!(is_umd_table(&with_colspan));
    }

    #[test]
    fn test_parse_simple_table() {
        // Without 'h' suffix, no thead should be generated
        let input = "| A | B |\n| C | D |";
        let html = parse_table(input);
        assert!(html.contains(r#"<table class="table umd-table">"#));
        assert!(!html.contains("<thead>"));
        assert!(html.contains("<tbody>"));
        assert!(html.contains("<td>A</td>"));
        assert!(html.contains("<td>C</td>"));
    }

    #[test]
    fn test_parse_table_with_header() {
        // With 'h' suffix, thead should be generated
        let input = "| ~A | ~B |h\n| C | D |";
        let html = parse_table(input);
        assert!(html.contains(r#"<table class="table umd-table">"#));
        assert!(html.contains("<thead>"));
        assert!(html.contains("<tbody>"));
        assert!(html.contains("<th>A</th>"));
        assert!(html.contains("<th>B</th>"));
        assert!(html.contains("<td>C</td>"));
        assert!(html.contains("<td>D</td>"));
    }

    #[test]
    fn test_parse_mixed_th_td() {
        // ~-prefixed cells become <th> even in body rows
        let input = "| A | B |h\n| ~Row Header | Data |";
        let html = parse_table(input);
        eprintln!("Output: {}", html);
        assert!(html.contains("<tbody>"));
        assert!(html.contains("<th>Row Header</th>"));
        assert!(html.contains("<td>Data</td>"));
    }

    #[test]
    fn test_parse_colspan() {
        let input = "| A |> |h\n| C | D |";
        let html = parse_table(input);
        eprintln!("Input: {}", input);
        eprintln!("Output: {}", html);
        assert!(html.contains("<thead>"));
        assert!(html.contains("<tbody>"));
        assert!(html.contains(r#"colspan="2""#));
    }

    #[test]
    fn test_parse_with_decoration() {
        let input = "| COLOR(red): ~A | B |h";
        let html = parse_table(input);
        eprintln!("Input: {}", input);
        eprintln!("Output: {}", html);
        // Bootstrap color names are output as classes
        assert!(html.contains("class="));
        assert!(html.contains("text-red"));
        // ~A becomes <th> with the color class
        assert!(html.contains(r#"<th class="text-red">A</th>"#));
    }
}
