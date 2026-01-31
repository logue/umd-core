//! UMD table support
//!
//! Provides support for UMD-style tables with extended features:
//! - Cell spanning: `|>` for colspan, `|^` for rowspan
//! - Cell decorations: COLOR(), SIZE(), alignment prefixes
//! - No mandatory header row (unlike GFM)

mod cell_spanning;
mod decorations;
mod parser;

// Re-export main API
pub use parser::{extract_umd_tables, parse_table};
