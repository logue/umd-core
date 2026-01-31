//! Table cell spanning support for LukiWiki
//!
//! Provides colspan and rowspan functionality for tables using special markers:
//! - `|>` for horizontal spanning (colspan)
//! - `|^` for vertical spanning (rowspan)

use once_cell::sync::Lazy;
use regex::Regex;

static TABLE_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?s)<table[^>]*>(.*?)</table>").unwrap());

static ROW_PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"<tr[^>]*>(.*?)</tr>").unwrap());

/// Cell information for tracking spans
#[derive(Debug, Clone)]
struct CellInfo {
    tag: String,        // "th" or "td"
    attributes: String, // existing attributes
    content: String,    // cell content
    colspan: usize,     // horizontal span count
    rowspan: usize,     // vertical span count
    is_merged: bool,    // true if this cell is merged into another
}

/// Parse table and apply cell spanning
///
/// # Arguments
///
/// * `html` - HTML content containing tables
///
/// # Returns
///
/// HTML with colspan and rowspan attributes applied
///
/// # Examples
///
/// ```
/// use universal_markdown::extensions::table_colspan::apply_table_colspan;
///
/// let input = r#"<table>
/// <tr><td>Cell1 |&gt;</td><td></td></tr>
/// </table>"#;
/// let output = apply_table_colspan(input);
/// assert!(output.contains(r#"colspan="2""#));
/// ```
pub fn apply_table_colspan(html: &str) -> String {
    TABLE_PATTERN
        .replace_all(html, |caps: &regex::Captures| {
            let full_match = caps.get(0).unwrap().as_str();

            // Skip LukiWiki tables (already processed)
            if full_match.contains(r#"data-lukiwiki="true""#) {
                return full_match.to_string();
            }

            let table_content = &caps[1];

            // Extract attributes from opening <table> tag
            let table_tag_end = full_match.find('>').unwrap();
            let opening_tag = &full_match[0..table_tag_end + 1];
            let table_attrs = opening_tag
                .trim_start_matches("<table")
                .trim_end_matches('>')
                .trim();

            let processed_content = process_table_content(table_content);

            if table_attrs.is_empty() {
                format!("<table>{}</table>", processed_content)
            } else {
                format!("<table {}>{}</table>", table_attrs, processed_content)
            }
        })
        .to_string()
}

/// Process table content to apply cell spanning
fn process_table_content(content: &str) -> String {
    // Extract all rows
    let mut rows: Vec<Vec<CellInfo>> = Vec::new();

    for row_cap in ROW_PATTERN.captures_iter(content) {
        let row_content = &row_cap[1];
        let mut cells: Vec<CellInfo> = Vec::new();

        // We need to preserve the order of cells as they appear in HTML
        // Find all <th> and <td> tags in order
        let cell_regex = Regex::new(r"<(th|td)([^>]*)>(.*?)</(?:th|td)>").unwrap();

        for cell_cap in cell_regex.captures_iter(row_content) {
            let tag = cell_cap[1].to_string();
            let attributes = cell_cap[2].to_string();
            let content = cell_cap[3].trim().to_string();

            cells.push(CellInfo {
                tag,
                attributes,
                content,
                colspan: 1,
                rowspan: 1,
                is_merged: false,
            });
        }

        rows.push(cells);
    }

    // Process colspan (horizontal spanning)
    for row in &mut rows {
        let mut i = 0;
        while i < row.len() {
            // Check if this cell has |> marker
            if row[i].content.ends_with(" |&gt;")
                || row[i].content.ends_with("|&gt;")
                || row[i].content == "|&gt;"
            {
                // Remove the marker from content
                row[i].content = row[i]
                    .content
                    .trim_end_matches("|&gt;")
                    .trim_end_matches(" |&gt;")
                    .trim()
                    .to_string();

                // Count consecutive empty cells or cells with only |> to merge
                let mut span_count = 1;
                let mut j = i + 1;
                while j < row.len() {
                    let next_content = row[j].content.trim();
                    // Check if next cell is empty or just contains |>
                    if next_content.is_empty() || next_content == "|&gt;" {
                        row[j].is_merged = true;
                        span_count += 1;
                        j += 1;
                    } else {
                        break;
                    }
                }

                row[i].colspan = span_count;
            }
            i += 1;
        }
    }

    // Process rowspan (vertical spanning)
    // We need to track the actual column index considering colspan
    let max_cols = rows.iter().map(|r| r.len()).max().unwrap_or(0);

    for logical_col in 0..max_cols {
        let mut row_idx = 0;
        while row_idx < rows.len() {
            // Find the cell at this logical column position
            let mut physical_col = 0;
            let mut col_counter = 0;

            while physical_col < rows[row_idx].len() && col_counter < logical_col {
                if !rows[row_idx][physical_col].is_merged {
                    col_counter += rows[row_idx][physical_col].colspan;
                }
                physical_col += 1;
            }

            // Check if this cell has |^ marker
            if physical_col < rows[row_idx].len()
                && !rows[row_idx][physical_col].is_merged
                && (rows[row_idx][physical_col].content == "|^"
                    || rows[row_idx][physical_col].content.trim() == "|^")
            {
                // This cell should merge with the cell above
                if row_idx > 0 {
                    rows[row_idx][physical_col].is_merged = true;

                    // Find the source cell in the row above at the same logical column
                    let mut source_row = row_idx - 1;
                    let mut source_col = physical_col;

                    // Find the actual cell in previous rows (in case it's already merged)
                    while source_row > 0 {
                        let mut prev_physical_col = 0;
                        let mut prev_col_counter = 0;

                        while prev_physical_col < rows[source_row].len()
                            && prev_col_counter < logical_col
                        {
                            if !rows[source_row][prev_physical_col].is_merged {
                                prev_col_counter += rows[source_row][prev_physical_col].colspan;
                            }
                            prev_physical_col += 1;
                        }

                        if prev_physical_col < rows[source_row].len()
                            && !rows[source_row][prev_physical_col].is_merged
                        {
                            source_col = prev_physical_col;
                            break;
                        }
                        source_row -= 1;
                    }

                    // Increment rowspan of the source cell
                    if source_col < rows[source_row].len() {
                        rows[source_row][source_col].rowspan += 1;
                    }
                }
            }
            row_idx += 1;
        }
    }

    // Rebuild HTML
    let mut result = String::new();

    for row in &rows {
        result.push_str("<tr>");
        for cell in row {
            if cell.is_merged {
                continue; // Skip merged cells
            }

            let mut attrs = cell.attributes.clone();

            if cell.colspan > 1 {
                attrs.push_str(&format!(r#" colspan="{}""#, cell.colspan));
            }

            if cell.rowspan > 1 {
                attrs.push_str(&format!(r#" rowspan="{}""#, cell.rowspan));
            }

            result.push_str(&format!(
                "<{tag}{attrs}>{content}</{tag}>",
                tag = cell.tag,
                attrs = attrs,
                content = cell.content
            ));
        }
        result.push_str("</tr>");
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_colspan_basic() {
        let input = r#"<table>
<tr><td>Cell1 |&gt;</td><td></td><td>Cell3</td></tr>
</table>"#;
        let output = apply_table_colspan(input);
        assert!(output.contains(r#"colspan="2""#));
        assert!(output.contains("Cell1"));
        assert!(output.contains("Cell3"));
        // Merged cell should not appear
        assert_eq!(output.matches("<td").count(), 2);
    }

    #[test]
    fn test_colspan_multiple() {
        let input = r#"<table>
<tr><td>Span3 |&gt;</td><td></td><td></td></tr>
</table>"#;
        let output = apply_table_colspan(input);
        assert!(output.contains(r#"colspan="3""#));
        assert_eq!(output.matches("<td").count(), 1);
    }

    #[test]
    fn test_rowspan_basic() {
        let input = r#"<table>
<tr><td>Cell1</td><td>Cell2</td></tr>
<tr><td>|^</td><td>Cell4</td></tr>
</table>"#;
        let output = apply_table_colspan(input);
        assert!(output.contains(r#"rowspan="2""#));
        assert!(output.contains("Cell1"));
        assert!(output.contains("Cell2"));
        assert!(output.contains("Cell4"));
        // First cell in second row should be merged
        let trs: Vec<_> = output.match_indices("<tr>").collect();
        assert_eq!(trs.len(), 2);
        let second_tr_start = trs[1].0 + 4;
        let second_tr = &output[second_tr_start..];
        let second_tr_cells = second_tr.split("</tr>").next().unwrap();
        assert_eq!(second_tr_cells.matches("<td").count(), 1); // Only Cell4
    }

    #[test]
    fn test_no_spanning() {
        let input = r#"<table>
<tr><td>Cell1</td><td>Cell2</td></tr>
<tr><td>Cell3</td><td>Cell4</td></tr>
</table>"#;
        let output = apply_table_colspan(input);
        assert!(!output.contains("colspan"));
        assert!(!output.contains("rowspan"));
        assert_eq!(output.matches("<td").count(), 4);
    }

    #[test]
    fn test_header_cells() {
        let input = r#"<table>
<tr><th>Header1 |&gt;</th><th></th></tr>
</table>"#;
        let output = apply_table_colspan(input);
        assert!(output.contains(r#"colspan="2""#));
        assert!(output.contains("<th"));
        assert_eq!(output.matches("<th").count(), 1);
    }

    #[test]
    fn test_with_classes() {
        let input = r#"<table class="table">
<tr><td class="text-center">Cell1 |&gt;</td><td></td></tr>
</table>"#;
        let output = apply_table_colspan(input);
        assert!(output.contains(r#"class="table""#));
        assert!(output.contains(r#"class="text-center""#));
        assert!(output.contains(r#"colspan="2""#));
    }
}
