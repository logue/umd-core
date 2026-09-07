//! GFM (standard Markdown) table handling: default class + restoring UMD
//! tables that were extracted and marker-protected by `table::umd` during
//! `conflict_resolver::preprocess_conflicts`.
//!
//! Table cell *alignment* prefixes (`V-START:` etc.) are a separate concern
//! handled by `alignment::process_table_cell_alignment` — see that module.

use regex::Regex;

/// Add the default `umd-list-table` class to every GFM `<table>` element.
///
/// No vertical column dividers — see `scss/components/table.scss`.
/// PukiWiki-style UMD tables (with `|>` colspan / `|^` rowspan) get their
/// own `umd-table` class directly in `table::umd::parser` instead, since
/// they're built there, not via comrak's bare `<table>`.
pub(crate) fn apply_default_class(html: &str) -> String {
    let table_pattern = Regex::new(r"<table>").unwrap();
    table_pattern
        .replace_all(html, "<table class=\"umd-list-table\">")
        .to_string()
}

/// Restore UMD tables that `table::umd::extract_umd_tables` pulled out
/// during preprocessing, keyed by the marker comrak wraps in a `<p>` tag
/// (and strips newlines from).
pub(crate) fn restore_markers(html: &str, tables: &[(String, String)]) -> String {
    let mut result = html.to_string();
    for (marker, table_html) in tables {
        let marker_text = marker.trim();
        let comrak_marker = format!("<p>{}</p>", marker_text);
        result = result.replace(&comrak_marker, table_html);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_default_class() {
        let input = "<table><tr><td>Cell</td></tr></table>";
        let output = apply_default_class(input);
        assert!(output.contains(r#"<table class="umd-list-table">"#));
    }

    #[test]
    fn test_restore_markers() {
        let tables = vec![(
            "{{TABLE:0}}".to_string(),
            "<table class=\"umd-table\"></table>".to_string(),
        )];
        let input = "<p>{{TABLE:0}}</p>";
        let output = restore_markers(input, &tables);
        assert_eq!(output, "<table class=\"umd-table\"></table>");
    }
}
