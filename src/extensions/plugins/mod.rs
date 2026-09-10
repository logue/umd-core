//! Plugin notation: `&function(args){content};` (inline) and
//! `@function(args){{content}}` / `:::function args ... :::` (block).
//!
//! Each direction owns both its "protect" step (turning the raw syntax into
//! a safe marker before Markdown parsing) and its "restore" step (turning
//! the marker back into HTML afterward) — see `inline` and `block`. This
//! module holds the small set of helpers shared by both directions.

pub mod block;
pub mod inline;

thread_local! {
    static MATH_CONVERTER: std::cell::RefCell<Option<math_core::LatexToMathML>> =
        std::cell::RefCell::new(
            math_core::LatexToMathML::new(math_core::MathCoreConfig::default()).ok()
        );
}

/// Escape HTML special characters
///
/// # Arguments
///
/// * `input` - Text to escape
///
/// # Returns
///
/// HTML-escaped string
fn escape_html_text(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Parse comma-separated args into a vector
///
/// # Arguments
///
/// * `args` - Comma-separated argument string
///
/// # Returns
///
/// Vector of trimmed argument strings
fn parse_args(args: &str) -> Vec<String> {
    if args.trim().is_empty() {
        return vec![];
    }
    args.split(',').map(|s| s.trim().to_string()).collect()
}

/// Render args as <data> elements
///
/// # Arguments
///
/// * `args` - Comma-separated argument string
///
/// # Returns
///
/// HTML string with <data value="index">arg</data> elements
fn render_args_as_data(args: &str) -> String {
    parse_args(args)
        .iter()
        .enumerate()
        .map(|(i, arg)| format!("<data value=\"{}\">{}</data>", i, escape_html_text(arg)))
        .collect::<Vec<_>>()
        .join("")
}

fn render_math_html(formula: &str, block_display: bool) -> Option<String> {
    let formula = formula.trim();
    if formula.is_empty() {
        return None;
    }

    let display = if block_display {
        math_core::MathDisplay::Block
    } else {
        math_core::MathDisplay::Inline
    };

    let converted = MATH_CONVERTER.with(|converter_cell| {
        let mut converter = converter_cell.borrow_mut();
        converter
            .as_mut()
            .and_then(|converter| converter.convert_with_local_state(formula, display).ok())
    });

    match converted {
        Some(result) => Some(result.mathml),
        None => Some(format!(
            "<span class=\"umd-math-error\" data-math-source=\"{}\">{}</span>",
            escape_html_text(formula),
            escape_html_text(formula)
        )),
    }
}

fn render_popover_html(trigger_text: &str, raw_content: &str) -> String {
    let popover_id = format!("umd-popover-{}", uuid::Uuid::new_v4().simple());
    let content_html = crate::parse(raw_content);
    format!(
        "<button command=\"show-popover\" commandfor=\"{}\">{}</button><div id=\"{}\" popover>{}</div>",
        popover_id,
        escape_html_text(trigger_text.trim()),
        popover_id,
        content_html
    )
}

/// Scan forward from an opening single-character delimiter (`chars[start]`
/// is assumed to be `open`) for its matching `close`, tracking nesting
/// depth so a balanced inner pair (e.g. a Markdown link's `(...)`  inside a
/// plugin's args, or a nested plugin call's `{...}` inside another's
/// content) doesn't prematurely end the scan.
///
/// Returns the index of the matching closing delimiter, or `None` if the
/// delimiters never balance before the end of input. Shared by
/// `inline::parse_inline_plugin_at` (for `(...)` args and `{...}` content)
/// and `block::parse_block_plugin_at` (for `(...)` args and single-line
/// `{...}` content) — see those for why a hand-written scanner replaced the
/// naive single-pass regexes that used to do this.
pub(super) fn scan_balanced(chars: &[char], start: usize, open: char, close: char) -> Option<usize> {
    let mut depth = 1i32;
    let mut i = start + 1;
    while i < chars.len() {
        if chars[i] == open {
            depth += 1;
        } else if chars[i] == close {
            depth -= 1;
            if depth == 0 {
                return Some(i);
            }
        }
        i += 1;
    }
    None
}

/// Like [`scan_balanced`], but for the two-character `{{` / `}}` delimiter
/// pair used by a block plugin's multiline content. `start` must be the
/// index of the first `{` of the opening `{{`. Returns the index of the
/// first `}` of the matching closing `}}`.
pub(super) fn scan_balanced_double(chars: &[char], start: usize) -> Option<usize> {
    let mut depth = 1i32;
    let mut i = start + 2;
    while i + 1 < chars.len() {
        if chars[i] == '{' && chars[i + 1] == '{' {
            depth += 1;
            i += 2;
        } else if chars[i] == '}' && chars[i + 1] == '}' {
            depth -= 1;
            if depth == 0 {
                return Some(i);
            }
            i += 2;
        } else {
            i += 1;
        }
    }
    None
}
