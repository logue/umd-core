//! Extended syntax for Universal Markdown
//!
//! This module provides extended syntax support including Bootstrap 5 integration,
//! semantic HTML elements, definition lists, and LukiWiki legacy compatibility.

pub mod alignment;
pub mod block_decoration;
pub mod conflict_resolver;
pub mod decoration_values;
pub mod fence;
pub mod inline;
pub mod media;
pub mod nested_blocks;
pub mod plugins;
pub mod preprocessor;
pub mod table;

/// Apply extended syntax transformations to HTML output
///
/// This function processes the HTML output from the Markdown parser and applies
/// extended syntax transformations including Bootstrap integration.
///
/// # Arguments
///
/// * `html` - The HTML output from the Markdown parser
///
/// # Returns
///
/// Transformed HTML with extended syntax applied
pub fn apply_extensions(html: &str) -> String {
    let header_map = conflict_resolver::HeaderIdMap::new();
    let options = crate::parser::ParserOptions::default();
    apply_extensions_with_headers(html, &header_map, &options)
}

/// Apply extended syntax transformations with custom header IDs
///
/// # Arguments
///
/// * `html` - The HTML output from the Markdown parser
/// * `header_map` - Map of custom header IDs
///
/// # Returns
///
/// Transformed HTML with extended syntax and custom header IDs applied
pub fn apply_extensions_with_headers(
    html: &str,
    header_map: &conflict_resolver::HeaderIdMap,
    options: &crate::parser::ParserOptions,
) -> String {
    let mut result = html.to_string();

    // Protect code blocks and inline code from transformation
    let (protected, placeholders) = fence::protect::protect_code_sections(&result);
    result = protected;

    // Apply transformations in order
    // Note: Plugins are handled in conflict_resolver::postprocess_conflicts
    result = media::transform_images_to_media(
        &result,
        &options.icons,
        options.allow_fragment_extension_hint,
    );
    result = conflict_resolver::postprocess_conflicts_with_options(
        &result,
        header_map,
        options.allow_hex_colors,
        options.allow_custom_font_size,
        &options.icons,
    );
    result = inline::emphasis::apply_umd_emphasis(&result);
    result = alignment::apply_block_placement(&result); // Apply block placement first
    result = block_decoration::apply_block_decorations_with_options(
        &result,
        options.allow_hex_colors,
        options.allow_custom_font_size,
    );
    // Second-pass inline plugin sweep (see plugins::inline docs), with plain
    // inline notation (%%text%%, ||text||) applied in between — matches the
    // original single-function's internal step order exactly.
    result = plugins::inline::prepare(&result, options.max_inline_nesting.map(usize::from));
    result = inline::notation::apply(&result);
    result = plugins::inline::expand(
        &result,
        options.allow_hex_colors,
        options.allow_custom_font_size,
    );

    // Apply base URL resolution to links
    if let Some(base_url) = &options.base_url {
        result = conflict_resolver::apply_base_url_to_links(&result, base_url);
    }

    // Restore protected code sections
    fence::protect::restore_code_sections(&result, &placeholders, &options.icons.color_swatch)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ParserOptions;

    #[test]
    fn test_umd_syntax_integration() {
        let input = "<p>This is ''bold'' and '''italic'''</p>";
        let output = apply_extensions(input);
        assert!(output.contains("<b>bold</b>"));
        assert!(output.contains("<i>italic</i>"));
    }

    #[test]
    fn test_inline_code_color_swatch_is_rendered_before_color_text() {
        let input = "<p><code>#FF5733</code></p>";
        let output = apply_extensions(input);
        assert!(output.contains(
            r#"<code><span class="inline-code-color" style="background-color: #FF5733;"><iconify-icon icon="ic:baseline-palette" aria-hidden="true"></iconify-icon></span>#FF5733</code>"#
        ));
    }

    #[test]
    fn test_inline_code_color_swatch_icon_is_configurable() {
        let input = "<p><code>#FF5733</code></p>";
        let header_map = conflict_resolver::HeaderIdMap::new();
        let mut options = ParserOptions::default();
        options.icons.color_swatch =
            r#"<span class="my-swatch-icon" aria-hidden="true"></span>"#.to_string();

        let output = apply_extensions_with_headers(input, &header_map, &options);
        assert!(output.contains(r#"<span class="my-swatch-icon" aria-hidden="true"></span>"#));
    }
}
