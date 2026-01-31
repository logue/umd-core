//! Cell spanning support for UMD tables
//!
//! Provides colspan and rowspan functionality using special markers:
//! - `|>` for horizontal spanning (colspan)
//! - `|^` for vertical spanning (rowspan)

use super::parser::Cell;

/// Process cell spanning (colspan and rowspan)
pub fn process_cell_spanning(rows: &mut Vec<Vec<Cell>>) {
    process_colspan(rows);
    process_rowspan(rows);
}

/// Process colspan (horizontal spanning)
fn process_colspan(rows: &mut Vec<Vec<Cell>>) {
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
}

/// Process rowspan (vertical spanning)
fn process_rowspan(rows: &mut Vec<Vec<Cell>>) {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_colspan() {
        let mut rows = vec![
            vec![
                Cell::new("A |>".to_string(), true),
                Cell::new("".to_string(), true),
            ],
            vec![
                Cell::new("C".to_string(), false),
                Cell::new("D".to_string(), false),
            ],
        ];

        process_cell_spanning(&mut rows);

        assert_eq!(rows[0].len(), 1);
        assert_eq!(rows[0][0].colspan, 2);
        assert_eq!(rows[0][0].content, "A");
    }

    #[test]
    fn test_rowspan() {
        let mut rows = vec![
            vec![
                Cell::new("A".to_string(), true),
                Cell::new("B".to_string(), true),
            ],
            vec![
                Cell::new("|^".to_string(), false),
                Cell::new("D".to_string(), false),
            ],
        ];

        process_cell_spanning(&mut rows);

        assert_eq!(rows[0][0].rowspan, 2);
        assert_eq!(rows[1].len(), 1);
        assert_eq!(rows[1][0].content, "D");
    }
}
