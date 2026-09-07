//! Position/alignment notation shared across paragraphs, tables, and block
//! plugins: `START`/`CENTER`/`END`/`JUSTIFY` (inline-direction) and
//! `V-START`/`V-CENTER`/`V-END`/`BASELINE` (block-direction).
//!
//! These are logical-direction names (matching `scss/utilities/text.scss`),
//! not physical `LEFT`/`RIGHT`/`TOP`/`BOTTOM` — under `dir="rtl"` or vertical
//! writing modes, "start" and "end" (or "block-start"/"block-end") flip
//! automatically, whereas a hardcoded `LEFT`/`TOP` would not. See PLAN.md's
//! "CSS 論理プロパティ方針".
//!
//! This module covers alignment wherever it appears:
//! - Paragraph-level prefixes, composed together with `COLOR()`/`SIZE()`/
//!   `TRUNCATE:` by `block_decoration::apply_block_decorations_with_options`
//!   (this module only supplies the alignment slice of that prefix chain).
//! - `apply_block_placement`: the `START:`/`CENTER:`/`END:`/`JUSTIFY:`
//!   wrapper placed on its own line before a table, block plugin, or media
//!   element.
//! - GFM table cells (`process_table_cell_alignment`), via `V-START:` etc.
//!   prefixes inside `<td>`/`<th>` content.
//! - UMD table cells (`strip_umd_table_alignment_prefix`), called from
//!   `table::umd::decorations`.

use once_cell::sync::Lazy;
use regex::{Captures, Regex};

/// Inline-direction alignment keyword → `umd-*` text-align class. The single
/// source of truth for this mapping — `ALIGN_EXTRACT`, `map_text_align`, and
/// (chained with `VERTICAL_ALIGN`) `CELL_ALIGNMENT_PREFIXES` all derive from
/// this table rather than each hardcoding the keyword list separately.
///
/// Order matters here only insofar as it fixes `CELL_ALIGNMENT_PREFIXES`'s
/// derived order (see that constant's docs) — kept as END/CENTER/START/
/// JUSTIFY to match the original hand-written table cell prefix list.
pub(crate) const TEXT_ALIGN: &[(&str, &str)] = &[
    ("END", "umd-end"),
    ("CENTER", "umd-center"),
    ("START", "umd-start"),
    ("JUSTIFY", "umd-justify"),
];

/// Block-direction alignment keyword → `umd-v-*` class. There's no CSS
/// logical equivalent of `vertical-align` — `V-` is UMD's own naming
/// convention. Single source of truth for this mapping, same rationale as
/// `TEXT_ALIGN` above.
pub(crate) const VERTICAL_ALIGN: &[(&str, &str)] = &[
    ("V-START", "umd-v-start"),
    ("V-CENTER", "umd-v-center"),
    ("V-END", "umd-v-end"),
    ("BASELINE", "umd-v-baseline"),
];

fn align_keywords_pattern(table: &[(&str, &str)]) -> String {
    let keywords: Vec<&str> = table.iter().map(|(keyword, _)| *keyword).collect();
    format!("({}):", keywords.join("|"))
}

pub(crate) static ALIGN_EXTRACT: Lazy<Regex> =
    Lazy::new(|| Regex::new(&align_keywords_pattern(TEXT_ALIGN)).unwrap());
pub(crate) static VALIGN_EXTRACT: Lazy<Regex> =
    Lazy::new(|| Regex::new(&align_keywords_pattern(VERTICAL_ALIGN)).unwrap());

// Block placement pattern for tables and plugins (must start on new line).
// Placement classes (`umd-block-*`) are a distinct namespace from the
// text-align classes in `TEXT_ALIGN` above (block positioning vs. inline
// text-align), so this keeps its own literal keyword list rather than
// reusing that table.
static BLOCK_PLACEMENT: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?m)^(START|CENTER|END|JUSTIFY):\n((?:\|[^\n]*\|(?:\n|$))+|@\w+(?:\([^)]*\))?\{[^}]*\})",
    )
    .unwrap()
});

fn lookup_align_class(table: &[(&str, &str)], value: &str, default: &str) -> String {
    let upper = value.to_uppercase();
    table
        .iter()
        .find(|(keyword, _)| *keyword == upper)
        .map(|(_, class)| class.to_string())
        .unwrap_or_else(|| default.to_string())
}

/// Map inline-direction alignment (`START`/`CENTER`/`END`/`JUSTIFY`) to a
/// `umd-*` text-align class
pub(crate) fn map_text_align(value: &str) -> String {
    lookup_align_class(TEXT_ALIGN, value, "umd-start")
}

/// Map block-direction alignment (`V-START`/`V-CENTER`/`V-END`/`BASELINE`)
/// to a `umd-v-*` class.
pub(crate) fn map_vertical_align(value: &str) -> String {
    lookup_align_class(VERTICAL_ALIGN, value, "umd-v-baseline")
}

/// UMD table cell alignment prefixes, in match order: `VERTICAL_ALIGN` then
/// `TEXT_ALIGN`, colon-suffixed. Used by
/// `table::umd::decorations::parse_cell_content`, which scans this list
/// once against the (progressively stripped) cell content — a later-in-list
/// prefix can be caught only after an earlier one strips, so the order here
/// (not just the keyword set) is significant and must stay
/// V-START/V-CENTER/V-END/BASELINE/END/CENTER/START/JUSTIFY.
pub(crate) fn cell_alignment_prefixes() -> impl Iterator<Item = (&'static str, &'static str)> {
    VERTICAL_ALIGN.iter().chain(TEXT_ALIGN.iter()).copied()
}

/// Merge extra classes into a tag's existing `class="..."` attribute (or add
/// one if absent), de-duplicating against classes already present. Shared by
/// `apply_block_placement` and `apply_pending_code_block_placement` — both
/// attach placement classes to an already-rendered element rather than
/// wrapping it in a new one.
pub(crate) fn merge_class_into_tag(tag_html: &str, extra_classes: &str) -> String {
    let class_re = Regex::new(r#"class=\"([^\"]*)\""#).unwrap();

    if let Some(caps) = class_re.captures(tag_html) {
        let existing = caps.get(1).map_or("", |m| m.as_str());
        let mut merged: Vec<String> = if existing.trim().is_empty() {
            Vec::new()
        } else {
            existing.split_whitespace().map(|s| s.to_string()).collect()
        };

        for class_name in extra_classes.split_whitespace() {
            if !merged.iter().any(|c| c == class_name) {
                merged.push(class_name.to_string());
            }
        }

        class_re
            .replace(tag_html, format!(r#"class="{}""#, merged.join(" ")))
            .to_string()
    } else {
        tag_html.replacen('>', &format!(r#" class="{}">"#, extra_classes), 1)
    }
}

/// Placement class for an element that is naturally full-width by default
/// (tables, code block figures) — START/CENTER/END need an explicit
/// shrink-to-content (`umd-block-auto`) before a margin utility can align it,
/// unlike media elements which shrink to their intrinsic size on their own.
fn placement_class_for_block(placement: &str) -> &'static str {
    match placement {
        "START" => "umd-block-auto",
        "CENTER" => "umd-block-center",
        "END" => "umd-block-auto umd-block-end",
        "JUSTIFY" => "umd-block-justify",
        _ => "",
    }
}

// A fenced code block interrupts the paragraph before it (unlike UMD's raw
// pipe-table/plugin syntax, which comrak treats as plain paragraph text), so
// by the time this module runs, code is still its `<!--CODE_BLOCK_n-->`
// placeholder (see `fence::protect`) sitting right after a sibling `<p>`, not
// nested inside it. This tags the placeholder with the requested placement so
// `apply_pending_code_block_placement` can apply it once the placeholder has
// become a real `<figure class="umd-code-block">` at the end of the pipeline.
static CODE_BLOCK_PLACEMENT: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?s)<p>\s*(START|CENTER|END|JUSTIFY):\s*</p>\s*(<!--CODE_BLOCK_\d+-->)"#).unwrap()
});

/// Merge a placement recorded by `CODE_BLOCK_PLACEMENT` into the
/// `<figure class="umd-code-block">` that the placeholder it wraps has since
/// become. Must run after `fence::code_block::process_code_blocks`.
pub(crate) fn apply_pending_code_block_placement(html: &str) -> String {
    static PENDING: Lazy<Regex> = Lazy::new(|| {
        Regex::new(
            r#"(?s)<div data-umd-code-placement="(START|CENTER|END|JUSTIFY)">\s*(<figure\b[^>]*\bclass="[^"]*\bumd-code-block\b[^"]*"[^>]*>[\s\S]*?</figure>)\s*</div>"#,
        )
        .unwrap()
    });

    PENDING
        .replace_all(html, |caps: &Captures| {
            let placement = &caps[1];
            let figure = &caps[2];
            merge_class_into_tag(figure, placement_class_for_block(placement))
        })
        .to_string()
}

/// Apply block placement prefixes to tables, block plugins, media, and code
/// blocks.
///
/// Handles START:/CENTER:/END:/JUSTIFY: prefixes followed by newline
/// for UMD tables and block plugins (@function), and (via
/// `CODE_BLOCK_PLACEMENT`/`apply_pending_code_block_placement`) preceding a
/// fenced code block. Logical-direction names (matching
/// block_decoration.rs's paragraph alignment and scss/utilities/text.scss),
/// not physical LEFT:/RIGHT:.
///
/// # Arguments
///
/// * `html` - The HTML content to process
///
/// # Returns
///
/// HTML with block placement applied (`umd-*` reference-CSS classes)
pub fn apply_block_placement(html: &str) -> String {
    let with_code_block_placement = CODE_BLOCK_PLACEMENT
        .replace_all(html, |caps: &Captures| {
            format!(
                r#"<div data-umd-code-placement="{}">{}</div>"#,
                &caps[1], &caps[2]
            )
        })
        .to_string();

    let media_block_placement = Regex::new(
        r#"(?s)<p>\s*(START|CENTER|END|JUSTIFY):\s*\n\s*(<picture[\s\S]*?</picture>|<video[\s\S]*?</video>|<audio[\s\S]*?</audio>|<a href="[^"]+" download class="download-link[^"]*"[^>]*>[\s\S]*?</a>)\s*</p>"#,
    )
    .unwrap();

    let with_media_placement = media_block_placement
        .replace_all(&with_code_block_placement, |caps: &regex::Captures| {
            let placement = &caps[1];
            let media = &caps[2];

            let wrapper_class = match placement {
                "START" => "umd-block-start",
                "CENTER" => "umd-inline-center",
                "END" => "umd-block-end",
                "JUSTIFY" => "umd-block-justify",
                _ => "",
            };

            if wrapper_class.is_empty() {
                format!("<figure>\n{}\n</figure>", media)
            } else {
                format!("<figure class=\"{}\">\n{}\n</figure>", wrapper_class, media)
            }
        })
        .to_string();

    let table_and_plugin_placement_in_paragraph = Regex::new(
        r#"(?s)<p>\s*(START|CENTER|END|JUSTIFY):\s*\n\s*(<(?:table|template)\b[^>]*>[\s\S]*?</(?:table|template)>)\s*</p>"#,
    )
    .unwrap();

    let with_table_and_plugin_placement_in_paragraph = table_and_plugin_placement_in_paragraph
        .replace_all(&with_media_placement, |caps: &regex::Captures| {
            let placement = &caps[1];
            let block = &caps[2];
            let placement_class = placement_class_for_block(placement);

            if block.starts_with("<table") {
                return merge_class_into_tag(block, placement_class);
            }

            if block.starts_with("<template") && block.contains("umd-plugin") {
                return merge_class_into_tag(block, placement_class);
            }

            block.to_string()
        })
        .to_string();

    let table_and_plugin_placement = Regex::new(
        r#"(?s)<p>\s*(START|CENTER|END|JUSTIFY):\s*</p>\s*(<(?:table|template)\b[^>]*>[\s\S]*?</(?:table|template)>)"#,
    )
    .unwrap();

    let with_table_and_plugin_placement = table_and_plugin_placement
        .replace_all(
            &with_table_and_plugin_placement_in_paragraph,
            |caps: &regex::Captures| {
                let placement = &caps[1];
                let block = &caps[2];
                let placement_class = placement_class_for_block(placement);

                if block.starts_with("<table") {
                    return merge_class_into_tag(block, placement_class);
                }

                if block.starts_with("<template") && block.contains("umd-plugin") {
                    return merge_class_into_tag(block, placement_class);
                }

                block.to_string()
            },
        )
        .to_string();

    BLOCK_PLACEMENT
        .replace_all(
            &with_table_and_plugin_placement,
            |caps: &regex::Captures| {
                let placement = &caps[1];
                let content = &caps[2];

                let wrapper_class = placement_class_for_block(placement);

                // Wrap table or plugin in div with appropriate class
                if content.starts_with('|') {
                    // UMD table
                    format!("<div class=\"{}\">\n{}</div>", wrapper_class, content)
                } else if content.starts_with('@') {
                    // Block plugin
                    format!("<div class=\"{}\">\n{}</div>", wrapper_class, content)
                } else {
                    content.to_string()
                }
            },
        )
        .to_string()
}

/// Process table cell vertical alignment prefixes (V-START:, V-CENTER:, V-END:, BASELINE:)
///
/// Detects alignment prefixes in table cells and adds `umd-v-*` alignment
/// classes (logical-direction names, matching block_decoration.rs and
/// scss/utilities/text.scss — not physical TOP:/BOTTOM:).
/// Note: GFM tables are handled by comrak without extensions.
/// UMD tables have their own cell spanning and decoration support.
pub(crate) fn process_table_cell_alignment(html: &str) -> String {
    let mut result = html.to_string();

    // Process <td> tags
    let td_pattern = Regex::new(r"<td([^>]*)>(.*?)</td>").unwrap();
    result = td_pattern
        .replace_all(&result, |caps: &Captures| {
            let existing_attrs = &caps[1];
            let content = &caps[2];
            process_cell_content("td", existing_attrs, content)
        })
        .to_string();

    // Process <th> tags
    let th_pattern = Regex::new(r"<th([^>]*)>(.*?)</th>").unwrap();
    result = th_pattern
        .replace_all(&result, |caps: &Captures| {
            let existing_attrs = &caps[1];
            let content = &caps[2];
            process_cell_content("th", existing_attrs, content)
        })
        .to_string();

    result
}

/// Process individual cell content for alignment
fn process_cell_content(tag: &str, existing_attrs: &str, content: &str) -> String {
    // Check for a vertical alignment prefix (GFM cells only support
    // vertical alignment prefixes — horizontal alignment for GFM tables
    // comes from the `|:---:|` delimiter-row syntax instead).
    let trimmed = content.trim_start();
    let matched = VERTICAL_ALIGN.iter().find_map(|(keyword, class)| {
        trimmed
            .strip_prefix(keyword)
            .and_then(|rest| rest.strip_prefix(':'))
            .map(|rest| (*class, rest.trim_start()))
    });

    match matched {
        None => format!("<{}{}>{}</{}>", tag, existing_attrs, content, tag),
        Some((align_class, remaining_content)) => {
            if existing_attrs.contains("class=") {
                // Append to existing class attribute
                let new_attrs =
                    existing_attrs.replace("class=\"", &format!("class=\"{} ", align_class));
                format!("<{}{}>{}</{}>", tag, new_attrs, remaining_content, tag)
            } else {
                // Add new class attribute
                format!(
                    "<{} class=\"{}\"{}>{}</{}>",
                    tag, align_class, existing_attrs, remaining_content, tag
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extensions::block_decoration::apply_block_decorations;

    #[test]
    fn test_text_align() {
        let input = "CENTER: Centered text";
        let output = apply_block_decorations(input);
        assert!(output.contains("class=\"umd-center\""));
    }

    #[test]
    fn test_text_align_start() {
        let input = "START: Start-aligned text";
        let output = apply_block_decorations(input);
        assert!(output.contains("class=\"umd-start\""));
    }

    #[test]
    fn test_text_align_end() {
        let input = "END: End-aligned text";
        let output = apply_block_decorations(input);
        assert!(output.contains("class=\"umd-end\""));
    }

    #[test]
    fn test_vertical_align() {
        let input = "V-START: Top aligned";
        let output = apply_block_decorations(input);
        assert!(output.contains("class=\"umd-v-start\""));
    }

    #[test]
    fn test_vertical_align_center_and_end() {
        assert!(
            apply_block_decorations("V-CENTER: Middle aligned").contains("class=\"umd-v-center\"")
        );
        assert!(apply_block_decorations("V-END: Bottom aligned").contains("class=\"umd-v-end\""));
    }

    #[test]
    fn test_vertical_align_baseline() {
        let input = "BASELINE: Baseline aligned";
        let output = apply_block_decorations(input);
        assert!(output.contains("class=\"umd-v-baseline\""));
    }

    #[test]
    fn test_block_placement_start() {
        let input = "START:\n|Header|\n|Cell|";
        let output = apply_block_placement(input);
        assert!(output.contains(r#"<div class="umd-block-auto">"#));
        assert!(output.contains("|Header|"));
    }

    #[test]
    fn test_block_placement_center() {
        let input = "CENTER:\n|Header|\n|Cell|";
        let output = apply_block_placement(input);
        assert!(output.contains(r#"<div class="umd-block-center">"#));
    }

    #[test]
    fn test_block_placement_end() {
        let input = "END:\n|Header|\n|Cell|";
        let output = apply_block_placement(input);
        assert!(output.contains(r#"<div class="umd-block-auto umd-block-end">"#));
    }

    #[test]
    fn test_block_placement_justify() {
        let input = "JUSTIFY:\n|Header|\n|Cell|";
        let output = apply_block_placement(input);
        assert!(output.contains(r#"<div class="umd-block-justify">"#));
    }

    #[test]
    fn test_block_placement_plugin() {
        let input = "CENTER:\n@youtube{video_id}";
        let output = apply_block_placement(input);
        assert!(output.contains(r#"<div class="umd-block-center">"#));
        assert!(output.contains("@youtube"));
    }

    #[test]
    fn test_block_placement_end_media() {
        let input = r#"<p>END:
<picture>
  <img src="image.png" alt="alt" title="Title" />
</picture></p>"#;
        let output = apply_block_placement(input);
        assert!(output.contains(r#"<figure class="umd-block-end">"#));
        assert!(output.contains("<picture>"));
        assert!(!output.contains("END:"));
    }

    #[test]
    fn test_table_cell_vertical_alignment() {
        let input = r#"<table class="table"><tr><td>V-START: Cell1</td><td>V-CENTER: Cell2</td></tr></table>"#;
        let output = process_table_cell_alignment(input);
        assert!(output.contains(r#"class="umd-v-start""#));
        assert!(output.contains("Cell1"));
        assert!(output.contains(r#"class="umd-v-center""#));
        assert!(output.contains("Cell2"));
    }

    #[test]
    fn test_table_cell_multiple_alignments() {
        let input = r#"<table><tr><th>BASELINE: Header</th><td>V-END: Data</td></tr></table>"#;
        let output = process_table_cell_alignment(input);
        assert!(output.contains(r#"class="umd-v-baseline""#));
        assert!(output.contains(r#"class="umd-v-end""#));
    }

    #[test]
    fn test_cell_alignment_prefixes_cover_all_logical_directions() {
        let classes: Vec<&str> = cell_alignment_prefixes().map(|(_, c)| c).collect();
        assert!(classes.contains(&"umd-center"));
        assert!(classes.contains(&"umd-v-start"));
        assert!(classes.contains(&"umd-v-baseline"));
    }
}
