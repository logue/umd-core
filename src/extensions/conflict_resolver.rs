//! Syntax conflict resolution for UMD and Markdown
//!
//! This module coordinates the pre-processing and post-processing stages
//! to resolve conflicts between UMD and Markdown syntax: header IDs, custom
//! link attributes, IDN warnings, GFM alerts, indeterminate task list state,
//! and base URL resolution. Scope-specific protect/restore steps (inline and
//! block plugins, block decorations, table markers, cell alignment) live in
//! their own modules and are only *called* from here, in pipeline order.

use once_cell::sync::Lazy;
use regex::{Captures, Regex};
use std::collections::HashMap;

use super::{alignment, block_decoration, plugins, preprocessor, table};

// Patterns that need special handling

/// Regex to detect Markdown-style emphasis that might conflict with UMD
/// Detects ***text*** which could be confused with '''text'''
static TRIPLE_STAR_EMPHASIS: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\*\*\*([^*]+)\*\*\*").unwrap());

/// Regex to detect custom header ID syntax: # Header {#custom-id}
static CUSTOM_HEADER_ID: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?m)^(#{1,6})\s+(.+?)\s+\{#([a-zA-Z0-9_-]+)\}\s*$").unwrap());

/// Store custom header IDs and UMD tables during preprocessing
#[derive(Debug, Clone)]
pub struct HeaderIdMap {
    /// Maps heading number (1-based) to custom ID
    pub ids: HashMap<usize, String>,
    /// Maps table markers to HTML content
    pub tables: Vec<(String, String)>,
}

impl HeaderIdMap {
    pub fn new() -> Self {
        Self {
            ids: HashMap::new(),
            tables: Vec::new(),
        }
    }
}

/// Pre-process input to resolve conflicts before Markdown parsing
///
/// This function escapes or transforms syntax that would otherwise create
/// ambiguous parsing situations. It also extracts custom header IDs.
///
/// # Arguments
///
/// * `input` - The raw wiki markup input
///
/// # Returns
///
/// A tuple of (pre-processed markup, header ID map)
///
/// # Examples
///
/// ```
/// use umd::extensions::conflict_resolver::preprocess_conflicts;
///
/// let input = "> quote <";
/// let (output, _) = preprocess_conflicts(input);
/// // UMD blockquote is preserved
/// ```
pub fn preprocess_conflicts(input: &str) -> (String, HeaderIdMap) {
    // Step 1: Remove comments before any other processing
    let mut result = preprocessor::remove_comments(input);

    let mut header_map = HeaderIdMap::new();
    let mut heading_counter = 0;

    // Extract custom header IDs: # Header {#custom-id}
    result = CUSTOM_HEADER_ID
        .replace_all(&result, |caps: &Captures| {
            heading_counter += 1;
            let hashes = &caps[1];
            let title = &caps[2];
            let custom_id = &caps[3];

            // Store the custom ID for this heading
            header_map
                .ids
                .insert(heading_counter, custom_id.to_string());

            // Return the heading without the {#id} part
            format!("{} {}", hashes, title)
        })
        .to_string();

    // Protect UMD block decorations (COLOR, SIZE, TRUNCATE, alignment, vertical
    // alignment) and the table/plugin block-placement prefix. These will be
    // applied in post-processing.
    //
    // START/END/V-START/V-CENTER/V-END/BASELINE/JUSTIFY/CENTER are shared by
    // two systems that both use this gate: block_decorations.rs's logical-
    // direction paragraph alignment, and apply_block_placement's table/plugin
    // placement wrapper (see that module's docs — both now use the same
    // logical-direction vocabulary, no physical LEFT/RIGHT/TOP/BOTTOM left).
    let block_decoration_prefix = Regex::new(
        r"(?m)^((?:(?:SIZE\([^)]+\)|COLOR\([^)]*\)|TRUNCATE|V-START|V-CENTER|V-END|BASELINE|JUSTIFY|END|START|CENTER):[ \t]*)+.*)$",
    )
    .unwrap();
    result = block_decoration_prefix
        .replace_all(&result, |caps: &Captures| {
            use base64::{Engine as _, engine::general_purpose};
            let encoded = general_purpose::STANDARD.encode(caps[1].as_bytes());
            format!(
                "{{{{BLOCK_DECORATION_B64:{}:BLOCK_DECORATION_B64}}}}",
                encoded
            )
        })
        .to_string();

    // Protect colon block plugin syntax (`::: 記法`) first, so that any
    // `@`/`&` plugin syntax nested inside a `:::` block is swallowed as
    // opaque literal content instead of being parsed as a nested plugin.
    result = plugins::block::protect_colon_block_plugins(&result);

    // Protect inline and block plugin syntax
    result = plugins::inline::protect_inline_plugins(&result);
    result = plugins::block::protect_block_plugins(&result);

    // Extract and protect UMD tables (before definition lists)
    let (result, table_map) = table::umd::extract_umd_tables(&result);
    header_map.tables = table_map;

    // Process definition lists: :term|definition
    let result = preprocessor::process_definition_lists(&result);

    (result, header_map)
}

fn is_valid_link_attr_token(token: &str) -> bool {
    !token.is_empty()
        && token
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
}

fn parse_link_attribute_spec(spec: &str) -> (Option<String>, Vec<String>) {
    let mut id = None;
    let mut classes = Vec::new();

    for raw_token in spec.split_whitespace() {
        let token = raw_token.trim();
        if token.is_empty() {
            continue;
        }

        if let Some(stripped) = token.strip_prefix('#') {
            if is_valid_link_attr_token(stripped) {
                id = Some(stripped.to_string());
            }
            continue;
        }

        if let Some(stripped) = token.strip_prefix('.') {
            if is_valid_link_attr_token(stripped) {
                classes.push(stripped.to_string());
            }
            continue;
        }

        if id.is_none() {
            if is_valid_link_attr_token(token) {
                id = Some(token.to_string());
            }
        } else if is_valid_link_attr_token(token) {
            classes.push(token.to_string());
        }
    }

    (id, classes)
}

fn apply_custom_link_attributes(html: &str) -> String {
    let link_pattern =
        Regex::new(r#"(?s)<a\s+([^>]*\bhref=\"[^\"]+\"[^>]*)>(.*?)</a>\s*\{([^}]+)\}"#).unwrap();
    let class_pattern = Regex::new(r#"class=\"([^\"]*)\""#).unwrap();
    let id_pattern = Regex::new(r#"\bid=\"[^\"]*\""#).unwrap();

    link_pattern
        .replace_all(html, |caps: &Captures| {
            let mut attrs = caps[1].to_string();
            let content = &caps[2];
            let spec = &caps[3];

            let (id, classes) = parse_link_attribute_spec(spec);

            if let Some(id_value) = id {
                if !id_pattern.is_match(&attrs) {
                    attrs.push_str(&format!(" id=\"{}\"", id_value));
                }
            }

            if !classes.is_empty() {
                if let Some(class_caps) = class_pattern.captures(&attrs) {
                    let existing = class_caps.get(1).map_or("", |m| m.as_str());
                    let mut class_list: Vec<String> =
                        existing.split_whitespace().map(|s| s.to_string()).collect();
                    for class_name in classes {
                        if !class_list.iter().any(|c| c == &class_name) {
                            class_list.push(class_name);
                        }
                    }
                    let merged = class_list.join(" ");
                    attrs = class_pattern
                        .replace(&attrs, format!("class=\"{}\"", merged))
                        .to_string();
                } else {
                    attrs.push_str(&format!(" class=\"{}\"", classes.join(" ")));
                }
            }

            format!("<a {}>{}</a>", attrs, content)
        })
        .to_string()
}

fn extract_http_host(href: &str) -> Option<&str> {
    let rest = if let Some(stripped) = href.strip_prefix("http://") {
        stripped
    } else if let Some(stripped) = href.strip_prefix("https://") {
        stripped
    } else {
        return None;
    };

    let authority_end = rest
        .find(|c| ['/', '?', '#'].contains(&c))
        .unwrap_or(rest.len());
    let authority = &rest[..authority_end];
    if authority.is_empty() {
        return None;
    }

    let host_port = authority
        .rsplit_once('@')
        .map_or(authority, |(_, host)| host);
    if host_port.is_empty() {
        return None;
    }

    if host_port.starts_with('[') {
        let end = host_port.find(']')?;
        return Some(&host_port[..=end]);
    }

    let host = host_port.split(':').next().unwrap_or("");
    if host.is_empty() { None } else { Some(host) }
}

fn is_idn_host(host: &str) -> bool {
    if host.starts_with('[') {
        return false;
    }

    let has_non_ascii = host.chars().any(|c| !c.is_ascii());
    let has_percent_encoded = host.contains('%');
    let has_punycode_label = host
        .split('.')
        .any(|label| label.to_ascii_lowercase().starts_with("xn--"));

    has_non_ascii || has_percent_encoded || has_punycode_label
}

fn should_warn_idn_link(href: &str) -> bool {
    extract_http_host(href).is_some_and(is_idn_host)
}

fn apply_idn_link_warnings(html: &str) -> String {
    let link_pattern =
        Regex::new(r#"(?s)<a\s+([^>]*\bhref=(?:\"([^\"]+)\"|'([^']+)')[^>]*)>(.*?)</a>"#).unwrap();
    let class_double_pattern = Regex::new(r#"class=\"([^\"]*)\""#).unwrap();
    let class_single_pattern = Regex::new(r"class='([^']*)'").unwrap();

    link_pattern
        .replace_all(html, |caps: &Captures| {
            let mut attrs = caps[1].to_string();
            let href = caps.get(2).or_else(|| caps.get(3)).map_or("", |m| m.as_str());
            let mut content = caps[4].to_string();

            if !should_warn_idn_link(href) {
                return caps[0].to_string();
            }

            if let Some(class_caps) = class_double_pattern.captures(&attrs) {
                let existing = class_caps.get(1).map_or("", |m| m.as_str());
                let mut class_list: Vec<String> =
                    existing.split_whitespace().map(|s| s.to_string()).collect();
                if !class_list.iter().any(|c| c == "umd-idn-warning-link") {
                    class_list.push("umd-idn-warning-link".to_string());
                }
                let merged = class_list.join(" ");
                attrs = class_double_pattern
                    .replace(&attrs, format!("class=\"{}\"", merged))
                    .to_string();
            } else if let Some(class_caps) = class_single_pattern.captures(&attrs) {
                let existing = class_caps.get(1).map_or("", |m| m.as_str());
                let mut class_list: Vec<String> =
                    existing.split_whitespace().map(|s| s.to_string()).collect();
                if !class_list.iter().any(|c| c == "umd-idn-warning-link") {
                    class_list.push("umd-idn-warning-link".to_string());
                }
                let merged = class_list.join(" ");
                attrs = class_single_pattern
                    .replace(&attrs, format!("class='{}'", merged))
                    .to_string();
            } else {
                attrs.push_str(" class=\"umd-idn-warning-link\"");
            }

            if !attrs.contains("data-idn-warning=") {
                attrs.push_str(" data-idn-warning=\"true\"");
            }

            if !content.contains("umd-idn-warning-icon") {
                content.push_str(
                    " <span class=\"umd-idn-warning-icon\" role=\"img\" aria-label=\"Internationalized domain warning\">&#9888;</span>",
                );
            }

            format!("<a {}>{}</a>", attrs, content)
        })
        .to_string()
}

pub fn postprocess_conflicts_with_options(
    html: &str,
    header_map: &HeaderIdMap,
    allow_hex_colors: bool,
    allow_custom_font_size: bool,
    icons: &crate::parser::Icons,
) -> String {
    // First, unescape quotes within markers to allow proper JSON parsing
    // comrak escapes quotes in JSON within markers, so we need to restore them
    // but ONLY within marker boundaries to avoid XSS
    let result = html.to_string();

    // Helper function to unescape quotes only within markers
    let unescape_marker_quotes = |input: &str| -> String {
        let marker_patterns = vec![
            (
                r"\{\{DEFINITION_LIST:([^\}]+):DEFINITION_LIST\}\}",
                "{{DEFINITION_LIST:",
            ),
            (
                r"\{\{INLINE_PLUGIN:([^\}]+):INLINE_PLUGIN\}\}",
                "{{INLINE_PLUGIN:",
            ),
            (
                r"\{\{BLOCK_PLUGIN:([^\}]+):BLOCK_PLUGIN\}\}",
                "{{BLOCK_PLUGIN:",
            ),
            (
                r"\{\{BLOCK_PLUGIN_ARGSONLY:([^\}]+):BLOCK_PLUGIN_ARGSONLY\}\}",
                "{{BLOCK_PLUGIN_ARGSONLY:",
            ),
            (
                r"\{\{COLON_BLOCK_PLUGIN:([^\}]+):COLON_BLOCK_PLUGIN\}\}",
                "{{COLON_BLOCK_PLUGIN:",
            ),
            (
                r"\{\{INLINE_PLUGIN_ARGSONLY:([^\}]+):INLINE_PLUGIN_ARGSONLY\}\}",
                "{{INLINE_PLUGIN_ARGSONLY:",
            ),
            (
                r"\{\{INLINE_PLUGIN_NOARGS:([^\}]+):INLINE_PLUGIN_NOARGS\}\}",
                "{{INLINE_PLUGIN_NOARGS:",
            ),
        ];

        let mut result = input.to_string();
        for (pattern, _marker_start) in marker_patterns {
            let re = Regex::new(pattern).unwrap();
            result = re
                .replace_all(&result, |caps: &Captures| {
                    let content = &caps[0];
                    content.replace("&quot;", "\"")
                })
                .to_string();
        }
        result
    };

    let mut result = unescape_marker_quotes(&result);

    // Add header IDs: <h1>Title</h1> -> <h1><a href="#id" id="id"></a>Title</h1>
    let mut heading_counter = 0;
    let header_regex = Regex::new(r"<h([1-6])>([^<]+)</h([1-6])>").unwrap();
    result = header_regex
        .replace_all(&result, |caps: &Captures| {
            heading_counter += 1;
            let level = &caps[1];
            let title = &caps[2];
            let close_level = &caps[3];

            let id = if let Some(custom_id) = header_map.ids.get(&heading_counter) {
                // Add 'h-' prefix to custom IDs to avoid conflicts with system IDs
                format!("h-{}", custom_id)
            } else {
                // Auto-numbered IDs also use 'h-' prefix for consistency
                format!("h-{}", heading_counter)
            };

            format!(
                "<h{}><a href=\"#{}\" aria-hidden=\"true\" class=\"anchor\" id=\"{}\"></a>{}</h{}>",
                level, id, id, title, close_level
            )
        })
        .to_string();

    // Restore and apply block decorations
    let block_decoration_marker =
        Regex::new(r"\{\{BLOCK_DECORATION_B64:([A-Za-z0-9+/=]+):BLOCK_DECORATION_B64\}\}").unwrap();

    result = block_decoration_marker
        .replace_all(&result, |caps: &Captures| {
            use base64::{Engine as _, engine::general_purpose};
            let encoded = &caps[1];
            let decoration = general_purpose::STANDARD
                .decode(encoded.as_bytes())
                .ok()
                .and_then(|bytes| String::from_utf8(bytes).ok())
                .unwrap_or_else(|| encoded.to_string());
            // Multiline decorations (e.g., END:\n<media>) and standalone block placement
            // prefixes are handled later by apply_block_placement.
            let placement_only = Regex::new(r"^(START|CENTER|END|JUSTIFY):\s*$")
                .unwrap()
                .is_match(decoration.trim());

            if decoration.contains('\n') || placement_only {
                decoration
            } else {
                block_decoration::apply_block_decorations_with_options(
                    &decoration,
                    allow_hex_colors,
                    allow_custom_font_size,
                )
            }
        })
        .to_string();

    // Restore inline plugin markers (&function(...){...}; and its
    // argsonly/noargs siblings) — see plugins::inline for the restore order.
    result = plugins::inline::restore_markers(&result, allow_hex_colors, allow_custom_font_size);

    // Restore block plugin markers (@function(...){{...}} and `::: 記法`) —
    // see plugins::block for the restore order.
    result = plugins::block::restore_markers(&result);

    // Remove wrapping <p> tags around template plugins
    let wrapped_plugin =
        Regex::new(r#"<p>\s*(<template class="umd-plugin[^"]*"[^>]*>.*?</template>)\s*</p>"#)
            .unwrap();
    result = wrapped_plugin.replace_all(&result, "$1").to_string();

    // Remove wrapping <p> tags around clearfix blocks
    let wrapped_clearfix = Regex::new(r#"<p>\s*(<div class="clearfix"></div>)\s*</p>"#).unwrap();
    result = wrapped_clearfix.replace_all(&result, "$1").to_string();

    // Restore definition lists
    let definition_list_marker =
        Regex::new(r"\{\{DEFINITION_LIST:([\s\S]*?):DEFINITION_LIST\}\}").unwrap();
    result = definition_list_marker
        .replace_all(&result, |caps: &Captures| {
            let items_json = &caps[1];

            // Parse JSON to get items
            let items: Vec<(String, String)> = serde_json::from_str(items_json).unwrap_or_default();

            if items.is_empty() {
                return String::new();
            }

            let mut dl_html = String::from("<dl>");
            for (term, definition) in items {
                dl_html.push_str(&format!("<dt>{}</dt><dd>{}</dd>", term, definition));
            }
            dl_html.push_str("</dl>");
            dl_html
        })
        .to_string();

    // Remove wrapping <p> tags around definition lists
    let wrapped_dl = Regex::new(r"<p>\s*(<dl>.*?</dl>)\s*</p>").unwrap();
    result = wrapped_dl.replace_all(&result, "$1").to_string();

    // Apply custom link attributes: [text](url){id class}
    result = apply_custom_link_attributes(&result);

    // Add a visual warning marker for external links that use IDN or punycode hosts.
    result = apply_idn_link_warnings(&result);

    // Apply indeterminate task list markers before other HTML transforms
    result = apply_tasklist_indeterminate(&result);

    // Apply Bootstrap default classes, GFM alerts, and table cell alignment
    result = apply_bootstrap_enhancements(&result, &header_map, icons);

    result
}

pub fn postprocess_conflicts(html: &str, header_map: &HeaderIdMap) -> String {
    postprocess_conflicts_with_options(
        html,
        header_map,
        false,
        false,
        &crate::parser::Icons::default(),
    )
}

/// Apply indeterminate task list state to rendered checkboxes.
fn apply_tasklist_indeterminate(html: &str) -> String {
    let pattern =
        Regex::new(r#"<input([^>]*\btype=\"checkbox\"[^>]*)/?>\s*\{\{TASK_INDETERMINATE\}\}"#)
            .unwrap();

    pattern
        .replace_all(html, |caps: &Captures| {
            let mut attrs = caps[1].to_string();
            if !attrs.contains("data-task=") {
                attrs.push_str(" data-task=\"indeterminate\"");
            }
            if !attrs.contains("aria-checked=") {
                attrs.push_str(" aria-checked=\"mixed\"");
            }
            format!("<input{} />", attrs)
        })
        .to_string()
}

/// Apply reference-CSS enhancements to HTML
///
/// - Add default `umd-list-table` class to all <table> elements (GFM tables —
///   UMD/PukiWiki-style tables get `umd-table` directly in table/umd/parser.rs)
/// - Add default `umd-blockquote` class to all <blockquote> elements (except UMD-style)
/// - Convert GFM-alert-style blockquotes ([!NOTE], etc.) to UMD's own .umd-note-* markup
fn apply_bootstrap_enhancements(
    html: &str,
    header_map: &HeaderIdMap,
    icons: &crate::parser::Icons,
) -> String {
    let mut result = html.to_string();

    // Add default class to standard/GFM Markdown tables (table::gfm — see
    // that module for why UMD tables don't go through this path).
    result = table::gfm::apply_default_class(&result);

    // Add default class to blockquotes
    let blockquote_pattern = Regex::new(r#"<blockquote>"#).unwrap();
    result = blockquote_pattern
        .replace_all(&result, "<blockquote class=\"umd-blockquote\">")
        .to_string();

    // Handle GFM-alert-style blockquotes: > [!NOTE] etc.
    // These are rendered as <blockquote class="umd-blockquote"><p>[!NOTE] ...</p></blockquote>.
    // Output is UMD's own .umd-note-* taxonomy, not Bootstrap's .alert-* — a host app
    // may use Bootstrap's alert component itself, so UMD doesn't borrow its classes.
    // Keyword aliases (DANGER/SUCCESS/WARN/INFO) are resolved here; the stylesheet only
    // needs to style the canonical type each alias maps to.
    //
    // Partial Digital Publishing WAI-ARIA (DPUB-ARIA) role, where a type has a real
    // DPUB-ARIA counterpart: doc-notice (note), doc-tip (tip), doc-example (example),
    // each with its ARIA fallback role per the DPUB-ARIA spec's fallback mapping
    // (doc-notice/doc-tip -> note, doc-example -> region). The remaining types
    // (important/warning/caution/must/recommend/dont/never) have no DPUB-ARIA
    // equivalent, so they're left without a role attribute rather than forced
    // into "note" semantics that wouldn't actually describe them.
    let gfm_alert_pattern = Regex::new(
        r#"<blockquote class="umd-blockquote">\s*<p>\[!(NOTE|TIP|IMPORTANT|WARNING|CAUTION|DANGER|MUST|RECOMMEND|DONT|NEVER|SUCCESS|WARN|INFO|EXAMPLE)\]\s*(.*?)</p>\s*</blockquote>"#
    ).unwrap();

    result = gfm_alert_pattern
        .replace_all(&result, |caps: &Captures| {
            let alert_type = &caps[1];
            let content = &caps[2];

            let (note_type, label, role, icon) = match alert_type {
                "NOTE" | "SUCCESS" => ("note", "Note", " role=\"doc-notice note\"", &icons.note),
                "TIP" | "INFO" => ("tip", "Tip", " role=\"doc-tip note\"", &icons.tip),
                "IMPORTANT" => ("important", "Important", "", &icons.important),
                "WARNING" | "WARN" => ("warning", "Warning", "", &icons.warning),
                "CAUTION" | "DANGER" => ("caution", "Caution", "", &icons.caution),
                "MUST" => ("must", "Must", "", &icons.must),
                "RECOMMEND" => ("recommend", "Recommend", "", &icons.recommend),
                "DONT" => ("dont", "Don't", "", &icons.dont),
                "NEVER" => ("never", "Never", "", &icons.never),
                "EXAMPLE" => ("example", "Example", " role=\"doc-example region\"", &icons.example),
                _ => ("note", "Note", " role=\"doc-notice note\"", &icons.note),
            };

            format!(
                "<aside class=\"umd-note umd-note-{}\"{}><p class=\"umd-note-title\">{} {}</p><p>{}</p></aside>",
                note_type, role, icon, label, content
            )
        })
        .to_string();

    // Restore UMD tables (table::gfm — comrak wraps markers in <p> tags and
    // strips newlines)
    result = table::gfm::restore_markers(&result, &header_map.tables);

    // Process table cell vertical alignment prefixes (for GFM tables only —
    // UMD tables have their own cell decoration support, see table::umd)
    result = alignment::process_table_cell_alignment(&result);

    result
}

/// Check if input contains potentially ambiguous syntax
///
/// Used for diagnostics and warnings. Returns descriptions of
/// detected conflicts.
///
/// # Arguments
///
/// * `input` - The raw wiki markup input
///
/// # Returns
///
/// Vector of warning messages for ambiguous patterns
pub fn detect_ambiguous_syntax(input: &str) -> Vec<String> {
    let mut warnings = Vec::new();

    // Check for ***text*** which could be confused with '''text'''
    if TRIPLE_STAR_EMPHASIS.is_match(input) && input.contains("'''") {
        warnings.push(
            "Detected both ***text*** (Markdown) and '''text''' (UMD). \
             Consider using **text** for Markdown bold-italic."
                .to_string(),
        );
    }

    // Check for potential COLOR(): vs Markdown definition list conflict
    if input.contains("COLOR(") && input.contains("\n:") {
        warnings.push(
            "Detected COLOR() syntax near Markdown definition list. \
             Ensure proper spacing to avoid ambiguity."
                .to_string(),
        );
    }

    warnings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_markdown_blockquote_unchanged() {
        let input = "> Standard Markdown quote\n> Second line";
        let (output, _) = preprocess_conflicts(input);
        // Should NOT be converted (no closing <)
        assert_eq!(output, input);
    }

    #[test]
    fn test_custom_header_id() {
        let input = "# My Header {#custom-id}\n\nContent";
        let (output, header_map) = preprocess_conflicts(input);
        // Should extract the custom ID
        assert_eq!(header_map.ids.get(&1), Some(&"custom-id".to_string()));
        // Should remove {#custom-id} from the text
        assert!(!output.contains("{#custom-id}"));
        assert!(output.contains("# My Header"));
    }

    #[test]
    fn test_multiple_custom_header_ids() {
        let input = "# First {#first}\n\n## Second {#second}\n\n### Third";
        let (_output, header_map) = preprocess_conflicts(input);
        assert_eq!(header_map.ids.get(&1), Some(&"first".to_string()));
        assert_eq!(header_map.ids.get(&2), Some(&"second".to_string()));
        assert_eq!(header_map.ids.get(&3), None); // No custom ID for third
    }

    #[test]
    fn test_apply_custom_header_ids() {
        let mut header_map = HeaderIdMap::new();
        header_map.ids.insert(1, "my-custom-id".to_string());

        let html = "<h1>Header</h1>";
        let output = postprocess_conflicts(html, &header_map);

        assert!(output.contains("id=\"h-my-custom-id\""));
        assert!(output.contains("href=\"#h-my-custom-id\""));
        assert!(!output.contains("heading-1"));
    }

    #[test]
    fn test_sequential_header_ids() {
        let header_map = HeaderIdMap::new();
        let html = "<h1>First</h1><h2>Second</h2>";
        let output = postprocess_conflicts(html, &header_map);

        assert!(output.contains("id=\"h-1\""));
        assert!(output.contains("id=\"h-2\""));
    }

    #[test]
    fn test_detect_triple_emphasis_conflict() {
        let input = "***Markdown*** and '''UMD'''";
        let warnings = detect_ambiguous_syntax(input);
        assert!(!warnings.is_empty());
        assert!(warnings[0].contains("***text***"));
    }

    #[test]
    fn test_detect_color_definition_conflict() {
        let input = "COLOR(red): text\n: definition";
        let warnings = detect_ambiguous_syntax(input);
        assert!(!warnings.is_empty());
        assert!(warnings[0].contains("COLOR()"));
    }

    #[test]
    fn test_no_warnings_for_clean_syntax() {
        let input = "# Heading\n\n**Bold** and ''UMD bold''";
        let warnings = detect_ambiguous_syntax(input);
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_bootstrap_table_class() {
        let header_map = HeaderIdMap::new();
        let input = "<table><tr><td>Cell</td></tr></table>";
        let output = postprocess_conflicts(input, &header_map);
        assert!(output.contains(r#"<table class="umd-list-table">"#));
    }

    #[test]
    fn test_bootstrap_blockquote_class() {
        let header_map = HeaderIdMap::new();
        let input = "<blockquote><p>Quote</p></blockquote>";
        let output = postprocess_conflicts(input, &header_map);
        assert!(output.contains(r#"<blockquote class="umd-blockquote">"#));
    }

    #[test]
    fn test_gfm_alert_note() {
        let header_map = HeaderIdMap::new();
        let icons = crate::parser::Icons::default();
        let input =
            r#"<blockquote class="umd-blockquote"><p>[!NOTE] This is a note</p></blockquote>"#;
        let output = postprocess_conflicts(input, &header_map);
        assert!(
            output.contains(r#"<aside class="umd-note umd-note-note" role="doc-notice note">"#)
        );
        assert!(output.contains(&format!(
            r#"<p class="umd-note-title">{} Note</p>"#,
            icons.note
        )));
        assert!(output.contains("This is a note"));
    }

    #[test]
    fn test_gfm_alert_example() {
        let header_map = HeaderIdMap::new();
        let icons = crate::parser::Icons::default();
        let input =
            r#"<blockquote class="umd-blockquote"><p>[!EXAMPLE] Sample usage</p></blockquote>"#;
        let output = postprocess_conflicts(input, &header_map);
        assert!(
            output
                .contains(r#"<aside class="umd-note umd-note-example" role="doc-example region">"#)
        );
        assert!(output.contains(&format!(
            r#"<p class="umd-note-title">{} Example</p>"#,
            icons.example
        )));
    }

    #[test]
    fn test_gfm_alert_types_without_dpub_role_have_no_role_attr() {
        let header_map = HeaderIdMap::new();
        for input_type in [
            "IMPORTANT",
            "WARNING",
            "CAUTION",
            "MUST",
            "RECOMMEND",
            "DONT",
            "NEVER",
        ] {
            let input = format!(
                r#"<blockquote class="umd-blockquote"><p>[!{}] Body</p></blockquote>"#,
                input_type
            );
            let output = postprocess_conflicts(&input, &header_map);
            assert!(
                !output.contains(" role="),
                "expected no role attribute for [!{}]: {}",
                input_type,
                output
            );
        }
    }

    #[test]
    fn test_gfm_alert_warning() {
        let header_map = HeaderIdMap::new();
        let icons = crate::parser::Icons::default();
        let input =
            r#"<blockquote class="umd-blockquote"><p>[!WARNING] Be careful</p></blockquote>"#;
        let output = postprocess_conflicts(input, &header_map);
        assert!(output.contains(r#"<aside class="umd-note umd-note-warning">"#));
        assert!(output.contains(&format!(
            r#"<p class="umd-note-title">{} Warning</p>"#,
            icons.warning
        )));
    }

    #[test]
    fn test_gfm_alert_extended_types() {
        let header_map = HeaderIdMap::new();
        let icons = crate::parser::Icons::default();
        for (input_type, expected_class, expected_label, expected_icon) in [
            ("MUST", "umd-note-must", "Must", &icons.must),
            (
                "RECOMMEND",
                "umd-note-recommend",
                "Recommend",
                &icons.recommend,
            ),
            ("DONT", "umd-note-dont", "Don't", &icons.dont),
            ("NEVER", "umd-note-never", "Never", &icons.never),
        ] {
            let input = format!(
                r#"<blockquote class="umd-blockquote"><p>[!{}] Body</p></blockquote>"#,
                input_type
            );
            let output = postprocess_conflicts(&input, &header_map);
            assert!(
                output.contains(&format!("umd-note {}", expected_class)),
                "expected class {} in output for [!{}]: {}",
                expected_class,
                input_type,
                output
            );
            assert!(
                output.contains(&format!(
                    r#"<p class="umd-note-title">{} {}</p>"#,
                    expected_icon, expected_label
                )),
                "expected label {} in output for [!{}]: {}",
                expected_label,
                input_type,
                output
            );
        }
    }

    #[test]
    fn test_gfm_alert_aliases_map_to_canonical_type() {
        let header_map = HeaderIdMap::new();
        for (alias, canonical_class) in [
            ("DANGER", "umd-note-caution"),
            ("SUCCESS", "umd-note-note"),
            ("WARN", "umd-note-warning"),
            ("INFO", "umd-note-tip"),
        ] {
            let input = format!(
                r#"<blockquote class="umd-blockquote"><p>[!{}] Body</p></blockquote>"#,
                alias
            );
            let output = postprocess_conflicts(&input, &header_map);
            assert!(
                output.contains(&format!("umd-note {}", canonical_class)),
                "expected alias [!{}] to map to {}: {}",
                alias,
                canonical_class,
                output
            );
        }
    }

    #[test]
    fn test_definition_list() {
        let input = ":Term 1|Definition 1\n:Term 2|Definition 2";
        let (preprocessed, _) = preprocess_conflicts(input);
        assert!(preprocessed.contains("{{DEFINITION_LIST:"));
    }

    #[test]
    fn test_definition_list_html_output() {
        let header_map = HeaderIdMap::new();
        let input = ":HTML|HyperText Markup Language\n:CSS|Cascading Style Sheets";
        let (preprocessed, _) = preprocess_conflicts(input);
        let output = postprocess_conflicts(&preprocessed, &header_map);
        assert!(output.contains("<dl>"));
        assert!(output.contains("<dt>HTML</dt>"));
        assert!(output.contains("<dd>HyperText Markup Language</dd>"));
        assert!(output.contains("<dt>CSS</dt>"));
        assert!(output.contains("<dd>Cascading Style Sheets</dd>"));
        assert!(output.contains("</dl>"));
    }

    #[test]
    fn test_tasklist_indeterminate_marker() {
        let header_map = HeaderIdMap::new();
        let input = r#"<li><input type="checkbox" disabled="" /> {{TASK_INDETERMINATE}}Item</li>"#;
        let output = postprocess_conflicts(input, &header_map);
        assert!(output.contains(r#"data-task="indeterminate""#));
        assert!(output.contains(r#"aria-checked="mixed""#));
        assert!(!output.contains("{{TASK_INDETERMINATE}}"));
    }

    #[test]
    fn test_custom_link_attributes_id_and_class() {
        let header_map = HeaderIdMap::new();
        let input = r#"<p><a href="/docs">Docs</a>{docs-link btn btn-primary}</p>"#;
        let output = postprocess_conflicts(input, &header_map);

        assert!(
            output.contains(r#"<a href="/docs" id="docs-link" class="btn btn-primary">Docs</a>"#)
        );
        assert!(!output.contains("{docs-link btn btn-primary}"));
    }

    #[test]
    fn test_custom_link_attributes_merge_class() {
        let header_map = HeaderIdMap::new();
        let input = r#"<a href="/home" class="existing">Home</a>{home-link new}"#;
        let output = postprocess_conflicts(input, &header_map);

        assert!(output.contains(r#"id="home-link""#));
        assert!(output.contains(r#"class="existing new""#));
    }

    #[test]
    fn test_idn_warning_for_unicode_domain() {
        let header_map = HeaderIdMap::new();
        let input = r#"<a href="https://日本.jp">Site</a>"#;
        let output = postprocess_conflicts(input, &header_map);

        assert!(output.contains(r#"class="umd-idn-warning-link""#));
        assert!(output.contains(r#"data-idn-warning="true""#));
        assert!(output.contains(r#"class="umd-idn-warning-icon""#));
    }

    #[test]
    fn test_idn_warning_for_punycode_domain() {
        let header_map = HeaderIdMap::new();
        let input = r#"<a href="https://xn--wgv71a.jp">Site</a>"#;
        let output = postprocess_conflicts(input, &header_map);

        assert!(output.contains(r#"class="umd-idn-warning-link""#));
        assert!(output.contains(r#"data-idn-warning="true""#));
    }

    #[test]
    fn test_idn_warning_for_percent_encoded_host() {
        let header_map = HeaderIdMap::new();
        let input = r#"<a href="https://%E6%97%A5%E6%9C%AC.jp">Site</a>"#;
        let output = postprocess_conflicts(input, &header_map);

        assert!(output.contains(r#"class="umd-idn-warning-link""#));
        assert!(output.contains(r#"data-idn-warning="true""#));
    }

    #[test]
    fn test_no_idn_warning_for_ascii_domain() {
        let header_map = HeaderIdMap::new();
        let input = r#"<a href="https://example.com">Site</a>"#;
        let output = postprocess_conflicts(input, &header_map);

        assert!(!output.contains("umd-idn-warning-link"));
        assert!(!output.contains("data-idn-warning"));
        assert!(!output.contains("umd-idn-warning-icon"));
    }

    #[test]
    fn test_no_idn_warning_for_relative_link() {
        let header_map = HeaderIdMap::new();
        let input = r#"<a href="/docs">Docs</a>"#;
        let output = postprocess_conflicts(input, &header_map);

        assert!(!output.contains("umd-idn-warning-link"));
        assert!(!output.contains("data-idn-warning"));
    }
}

/// Apply base URL to absolute paths in links and media
///
/// Resolves absolute paths (starting with "/") by prefixing them with the base_url.
/// Relative URLs (http://, https://, //, etc.) are left unchanged.
///
/// # Arguments
///
/// * `html` - The HTML to process
/// * `base_url` - The base URL to prepend (e.g., "/app", "https://example.com/app")
///
/// # Returns
///
/// HTML with absolute paths resolved to base_url + path
///
/// # Examples
///
/// ```
/// use umd::extensions::conflict_resolver::apply_base_url_to_links;
///
/// let html = r#"<a href="/docs">Docs</a><img src="/image.png" />"#;
/// let result = apply_base_url_to_links(html, "/app");
/// assert!(result.contains(r#"href="/app/docs""#));
/// assert!(result.contains(r#"src="/app/image.png""#));
/// ```
pub fn apply_base_url_to_links(html: &str, base_url: &str) -> String {
    // Normalize base_url: remove trailing slash
    let normalized_base = if base_url.ends_with('/') && base_url.len() > 1 {
        &base_url[..base_url.len() - 1]
    } else {
        base_url
    };

    let mut result = html.to_string();

    // Replace href="/path" with href="/base_url/path"
    let href_double = Regex::new(r#"((?:href|src|srcset)\s*=\s*)"(/[^"]*)""#).unwrap();
    result = href_double
        .replace_all(&result, |caps: &Captures| {
            let attr = &caps[1];
            let path = &caps[2];
            let new_url = format!("{}{}", normalized_base, path);
            format!("{}\"{}\"", attr, new_url)
        })
        .to_string();

    // Replace href='/path' with href='/base_url/path' (single quotes)
    let href_single = Regex::new(r"((?:href|src|srcset)\s*=\s*)'(/[^']*)'").unwrap();
    result = href_single
        .replace_all(&result, |caps: &Captures| {
            let attr = &caps[1];
            let path = &caps[2];
            let new_url = format!("{}{}", normalized_base, path);
            format!("{}'{}'", attr, new_url)
        })
        .to_string();

    result
}
