//! Inline decoration functions for LukiWiki
//!
//! Provides inline formatting functions:
//! - &color(fg,bg){text};
//! - &size(rem){text};
//! - &sup(text); (superscript)
//! - &sub(text); (subscript)
//! - &lang(locale){text};
//! - &abbr(text){description};
//! - &ruby(reading){text}; (furigana)
//! - Semantic HTML elements: dfn, kbd, samp, var, cite, q, small, u
//! - &time(datetime){text};
//! - &data(value){text};
//! - &bdi(text); &bdo(dir){text};
//! - &wbr; (word break opportunity)
//! - %%text%% → <s>text</s> (strikethrough)

use once_cell::sync::Lazy;
use regex::Regex;

static INLINE_COLOR: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"&color\(([^,)]*?)(?:,([^)]*?))?\)\{([^}]+?)\};").unwrap());

static INLINE_SIZE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"&size\(([^)]+?)\)\{([^}]+?)\};").unwrap());

static INLINE_SUP: Lazy<Regex> = Lazy::new(|| Regex::new(r"&sup\(([^)]+?)\);").unwrap());

static INLINE_SUB: Lazy<Regex> = Lazy::new(|| Regex::new(r"&sub\(([^)]+?)\);").unwrap());

static INLINE_LANG: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"&lang\(([^)]+?)\)\{([^}]+?)\};").unwrap());

static INLINE_ABBR: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"&abbr\(([^)]+?)\)\{([^}]+?)\};").unwrap());

static INLINE_RUBY: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"&ruby\(([^)]+?)\)\{([^}]+?)\};").unwrap());

// Semantic HTML elements - simple wrapper tags
static INLINE_DFN: Lazy<Regex> = Lazy::new(|| Regex::new(r"&dfn\(([^)]+?)\);").unwrap());
static INLINE_KBD: Lazy<Regex> = Lazy::new(|| Regex::new(r"&kbd\(([^)]+?)\);").unwrap());
static INLINE_SAMP: Lazy<Regex> = Lazy::new(|| Regex::new(r"&samp\(([^)]+?)\);").unwrap());
static INLINE_VAR: Lazy<Regex> = Lazy::new(|| Regex::new(r"&var\(([^)]+?)\);").unwrap());
static INLINE_CITE: Lazy<Regex> = Lazy::new(|| Regex::new(r"&cite\(([^)]+?)\);").unwrap());
static INLINE_Q: Lazy<Regex> = Lazy::new(|| Regex::new(r"&q\(([^)]+?)\);").unwrap());
static INLINE_SMALL: Lazy<Regex> = Lazy::new(|| Regex::new(r"&small\(([^)]+?)\);").unwrap());
static INLINE_U: Lazy<Regex> = Lazy::new(|| Regex::new(r"&u\(([^)]+?)\);").unwrap());

// Elements with attributes
static INLINE_TIME: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"&time\(([^)]+?)\)\{([^}]+?)\};").unwrap());
static INLINE_DATA: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"&data\(([^)]+?)\)\{([^}]+?)\};").unwrap());

// Bidirectional text
static INLINE_BDI: Lazy<Regex> = Lazy::new(|| Regex::new(r"&bdi\(([^)]+?)\);").unwrap());
static INLINE_BDO: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"&bdo\(([^)]+?)\)\{([^}]+?)\};").unwrap());

// Word break opportunity (self-closing)
static INLINE_WBR: Lazy<Regex> = Lazy::new(|| Regex::new(r"&wbr;").unwrap());

/// Regex for LukiWiki strikethrough: %%text%% → <s>text</s>
static LUKIWIKI_STRIKETHROUGH: Lazy<Regex> = Lazy::new(|| Regex::new(r"%%([^%]+)%%").unwrap());

/// Apply inline decoration functions to HTML
///
/// # Arguments
///
/// * `html` - The HTML content to process
///
/// # Returns
///
/// HTML with inline decorations applied
pub fn apply_inline_decorations(html: &str) -> String {
    let mut result = html.to_string();

    // Apply %%text%% → <s>text</s> (LukiWiki strikethrough)
    result = LUKIWIKI_STRIKETHROUGH
        .replace_all(&result, "<s>$1</s>")
        .to_string();

    // Apply &color(fg,bg){text};
    result = INLINE_COLOR
        .replace_all(&result, |caps: &regex::Captures| {
            let fg = caps.get(1).map_or("", |m| m.as_str().trim());
            let bg = caps.get(2).map_or("", |m| m.as_str().trim());
            let text = caps.get(3).map_or("", |m| m.as_str());

            let mut styles = Vec::new();
            if !fg.is_empty() && fg != "inherit" {
                styles.push(format!("color: {}", fg));
            }
            if !bg.is_empty() && bg != "inherit" {
                styles.push(format!("background-color: {}", bg));
            }

            if styles.is_empty() {
                text.to_string()
            } else {
                format!("<span style=\"{}\">{}</span>", styles.join("; "), text)
            }
        })
        .to_string();

    // Apply &size(rem){text};
    result = INLINE_SIZE
        .replace_all(&result, |caps: &regex::Captures| {
            let size = caps.get(1).map_or("", |m| m.as_str());
            let text = caps.get(2).map_or("", |m| m.as_str());
            format!("<span style=\"font-size: {}rem\">{}</span>", size, text)
        })
        .to_string();

    // Apply &sup(text);
    result = INLINE_SUP
        .replace_all(&result, "<sup>$1</sup>;")
        .to_string();

    // Apply &sub(text);
    result = INLINE_SUB
        .replace_all(&result, "<sub>$1</sub>;")
        .to_string();

    // Apply &lang(locale){text};
    result = INLINE_LANG
        .replace_all(&result, "<span lang=\"$1\">$2</span>;")
        .to_string();

    // Apply &abbr(text){description};
    result = INLINE_ABBR
        .replace_all(&result, "<abbr title=\"$2\">$1</abbr>;")
        .to_string();

    // Apply &ruby(reading){text};
    result = INLINE_RUBY
        .replace_all(&result, "<ruby>$2<rp>(</rp><rt>$1</rt><rp>)</rp></ruby>;")
        .to_string();

    // Semantic HTML elements - simple wrappers
    result = INLINE_DFN
        .replace_all(&result, "<dfn>$1</dfn>;")
        .to_string();
    result = INLINE_KBD
        .replace_all(&result, "<kbd>$1</kbd>;")
        .to_string();
    result = INLINE_SAMP
        .replace_all(&result, "<samp>$1</samp>;")
        .to_string();
    result = INLINE_VAR
        .replace_all(&result, "<var>$1</var>;")
        .to_string();
    result = INLINE_CITE
        .replace_all(&result, "<cite>$1</cite>;")
        .to_string();
    result = INLINE_Q.replace_all(&result, "<q>$1</q>;").to_string();
    result = INLINE_SMALL
        .replace_all(&result, "<small>$1</small>;")
        .to_string();
    result = INLINE_U.replace_all(&result, "<u>$1</u>;").to_string();

    // Elements with attributes
    result = INLINE_TIME
        .replace_all(&result, "<time datetime=\"$1\">$2</time>;")
        .to_string();
    result = INLINE_DATA
        .replace_all(&result, "<data value=\"$1\">$2</data>;")
        .to_string();

    // Bidirectional text
    result = INLINE_BDI
        .replace_all(&result, "<bdi>$1</bdi>;")
        .to_string();
    result = INLINE_BDO
        .replace_all(&result, "<bdo dir=\"$1\">$2</bdo>;")
        .to_string();

    // Word break opportunity
    result = INLINE_WBR.replace_all(&result, "<wbr />").to_string();

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inline_color_foreground() {
        let input = "This is &color(red){red text};";
        let output = apply_inline_decorations(input);
        assert!(output.contains("<span style=\"color: red\">red text</span>"));
    }

    #[test]
    fn test_inline_color_background() {
        let input = "&color(,yellow){yellow bg};";
        let output = apply_inline_decorations(input);
        assert!(output.contains("<span style=\"background-color: yellow\">yellow bg</span>"));
    }

    #[test]
    fn test_inline_color_both() {
        let input = "&color(white,black){white on black};";
        let output = apply_inline_decorations(input);
        assert!(output.contains("color: white"));
        assert!(output.contains("background-color: black"));
    }

    #[test]
    fn test_inline_size() {
        let input = "&size(1.5){larger};";
        let output = apply_inline_decorations(input);
        assert!(output.contains("<span style=\"font-size: 1.5rem\">larger</span>"));
    }

    #[test]
    fn test_inline_sup() {
        let input = "x&sup(2);";
        let output = apply_inline_decorations(input);
        assert_eq!(output, "x<sup>2</sup>;");
    }

    #[test]
    fn test_inline_sub() {
        let input = "H&sub(2);O";
        let output = apply_inline_decorations(input);
        assert_eq!(output, "H<sub>2</sub>;O");
    }

    #[test]
    fn test_inline_lang() {
        let input = "&lang(en){Hello};";
        let output = apply_inline_decorations(input);
        assert_eq!(output, "<span lang=\"en\">Hello</span>;");
    }

    #[test]
    fn test_inline_abbr() {
        let input = "&abbr(HTML){HyperText Markup Language};";
        let output = apply_inline_decorations(input);
        assert_eq!(
            output,
            "<abbr title=\"HyperText Markup Language\">HTML</abbr>;"
        );
    }

    #[test]
    fn test_multiple_inline_decorations() {
        let input = "&color(red){Red}; and &size(2){Big}; and &sup(superscript);";
        let output = apply_inline_decorations(input);
        assert!(output.contains("color: red"));
        assert!(output.contains("font-size: 2rem"));
        assert!(output.contains("<sup>superscript</sup>"));
    }

    #[test]
    fn test_lukiwiki_strikethrough() {
        let input = "This is %%strikethrough%% text.";
        let output = apply_inline_decorations(input);
        assert_eq!(output, "This is <s>strikethrough</s> text.");
    }

    #[test]
    fn test_lukiwiki_strikethrough_multiple() {
        let input = "%%first%% and %%second%%";
        let output = apply_inline_decorations(input);
        assert_eq!(output, "<s>first</s> and <s>second</s>");
    }

    #[test]
    fn test_inline_ruby() {
        let input = "&ruby(Ashita){明日};";
        let output = apply_inline_decorations(input);
        assert_eq!(
            output,
            "<ruby>明日<rp>(</rp><rt>Ashita</rt><rp>)</rp></ruby>;"
        );
    }

    #[test]
    fn test_semantic_elements() {
        let input = "&dfn(term); &kbd(Ctrl); &samp(output); &var(x);";
        let output = apply_inline_decorations(input);
        assert!(output.contains("<dfn>term</dfn>"));
        assert!(output.contains("<kbd>Ctrl</kbd>"));
        assert!(output.contains("<samp>output</samp>"));
        assert!(output.contains("<var>x</var>"));
    }

    #[test]
    fn test_cite_q_small_u() {
        let input = "&cite(Book Title); &q(quote); &small(note); &u(underline);";
        let output = apply_inline_decorations(input);
        assert!(output.contains("<cite>Book Title</cite>"));
        assert!(output.contains("<q>quote</q>"));
        assert!(output.contains("<small>note</small>"));
        assert!(output.contains("<u>underline</u>"));
    }

    #[test]
    fn test_time_and_data() {
        let input = "&time(2026-01-26){today}; &data(12345){value};";
        let output = apply_inline_decorations(input);
        assert!(output.contains("<time datetime=\"2026-01-26\">today</time>"));
        assert!(output.contains("<data value=\"12345\">value</data>"));
    }

    #[test]
    fn test_bidirectional_text() {
        let input = "&bdi(مرحبا); &bdo(rtl){right-to-left};";
        let output = apply_inline_decorations(input);
        assert!(output.contains("<bdi>مرحبا</bdi>"));
        assert!(output.contains("<bdo dir=\"rtl\">right-to-left</bdo>"));
    }

    #[test]
    fn test_wbr() {
        let input = "Very&wbr;Long&wbr;Word";
        let output = apply_inline_decorations(input);
        assert_eq!(output, "Very<wbr />Long<wbr />Word");
    }
}
