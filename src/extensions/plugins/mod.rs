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
