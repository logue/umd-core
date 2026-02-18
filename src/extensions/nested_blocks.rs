//! Preprocess list items to allow nested block elements without extra indentation.
//!
//! CommonMark requires block elements inside list items to be indented. UMD allows
//! blocks like tables and code fences immediately after a list item, so we
//! normalize those blocks by adding indentation before comrak parses them.

use once_cell::sync::Lazy;
use regex::Regex;

static LIST_MARKER: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(?P<indent>[ \t]*)(?P<marker>(?:[-+*])|(?:\d+\.))\s+.+$").unwrap());

static PLACEMENT_PREFIX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(LEFT|CENTER|RIGHT|JUSTIFY):\s*$").unwrap());

/// Preprocess list items so nested block elements are indented properly.
pub fn preprocess_nested_blocks(input: &str) -> String {
    let lines: Vec<&str> = input.lines().collect();
    let mut output: Vec<String> = Vec::with_capacity(lines.len());
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];

        if let Some(list_indent) = list_indent_width(line) {
            output.push(line.to_string());
            i += 1;

            while i < lines.len() {
                let next_line = lines[i];
                if next_line.trim().is_empty() {
                    output.push(next_line.to_string());
                    i += 1;
                    continue;
                }

                if let Some(next_indent) = list_indent_width(next_line) {
                    if next_indent <= list_indent {
                        break;
                    }
                    output.push(next_line.to_string());
                    i += 1;
                    continue;
                }

                let next_indent = indent_width(next_line);
                if next_indent > list_indent {
                    output.push(next_line.to_string());
                    i += 1;
                    continue;
                }

                let target_indent = list_indent + 4;

                if is_table_line(next_line) {
                    i = indent_table_block(&lines, i, &mut output, target_indent);
                    continue;
                }

                if is_blockquote_line(next_line) {
                    i = indent_blockquote_block(&lines, i, &mut output, target_indent);
                    continue;
                }

                if is_code_fence_line(next_line).is_some() {
                    i = indent_code_fence_block(&lines, i, &mut output, target_indent);
                    continue;
                }

                if is_block_plugin_line(next_line) {
                    i = indent_plugin_block(&lines, i, &mut output, target_indent);
                    continue;
                }

                if is_block_placement_prefix(next_line)
                    && i + 1 < lines.len()
                    && (is_table_line(lines[i + 1]) || is_block_plugin_line(lines[i + 1]))
                {
                    output.push(indent_to(lines[i], target_indent));
                    i += 1;
                    continue;
                }

                output.push(next_line.to_string());
                i += 1;
            }
        } else {
            output.push(line.to_string());
            i += 1;
        }
    }

    output.join("\n")
}

fn list_indent_width(line: &str) -> Option<usize> {
    LIST_MARKER
        .captures(line)
        .and_then(|caps| caps.name("indent").map(|m| indent_width(m.as_str())))
}

fn indent_width(input: &str) -> usize {
    input
        .chars()
        .take_while(|ch| ch.is_whitespace())
        .map(|ch| if ch == '\t' { 4 } else { 1 })
        .sum()
}

fn indent_to(line: &str, target_indent: usize) -> String {
    let current_indent = indent_width(line);
    if current_indent >= target_indent {
        return line.to_string();
    }
    let padding = " ".repeat(target_indent - current_indent);
    format!("{}{}", padding, line)
}

fn is_table_line(line: &str) -> bool {
    line.trim_start().starts_with('|')
}

fn is_blockquote_line(line: &str) -> bool {
    line.trim_start().starts_with('>')
}

fn is_block_plugin_line(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.starts_with('@')
}

fn is_block_placement_prefix(line: &str) -> bool {
    PLACEMENT_PREFIX.is_match(line.trim_start())
}

fn is_code_fence_line(line: &str) -> Option<&'static str> {
    let trimmed = line.trim_start();
    if trimmed.starts_with("```") {
        Some("```")
    } else if trimmed.starts_with("~~~") {
        Some("~~~")
    } else {
        None
    }
}

fn indent_table_block(
    lines: &[&str],
    start: usize,
    output: &mut Vec<String>,
    target_indent: usize,
) -> usize {
    let mut i = start;
    while i < lines.len() && is_table_line(lines[i]) {
        output.push(indent_to(lines[i], target_indent));
        i += 1;
    }
    i
}

fn indent_blockquote_block(
    lines: &[&str],
    start: usize,
    output: &mut Vec<String>,
    target_indent: usize,
) -> usize {
    let mut i = start;
    while i < lines.len() && is_blockquote_line(lines[i]) {
        output.push(indent_to(lines[i], target_indent));
        i += 1;
    }
    i
}

fn indent_code_fence_block(
    lines: &[&str],
    start: usize,
    output: &mut Vec<String>,
    target_indent: usize,
) -> usize {
    let marker = match is_code_fence_line(lines[start]) {
        Some(marker) => marker,
        None => return start,
    };

    let mut i = start;
    output.push(indent_to(lines[i], target_indent));
    i += 1;

    while i < lines.len() {
        output.push(indent_to(lines[i], target_indent));
        if lines[i].trim_start().starts_with(marker) {
            i += 1;
            break;
        }
        i += 1;
    }

    i
}

fn indent_plugin_block(
    lines: &[&str],
    start: usize,
    output: &mut Vec<String>,
    target_indent: usize,
) -> usize {
    let mut i = start;
    let in_multiline = lines[i].contains("{{");

    output.push(indent_to(lines[i], target_indent));
    i += 1;

    if !in_multiline {
        return i;
    }

    while i < lines.len() {
        output.push(indent_to(lines[i], target_indent));
        if lines[i].contains("}}") {
            i += 1;
            break;
        }
        i += 1;
    }

    i
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_table_inside_list() {
        let input = "- Item\n| A | B |\n| C | D |";
        let output = preprocess_nested_blocks(input);
        assert!(output.contains("- Item\n    | A | B |\n    | C | D |"));
    }

    #[test]
    fn test_code_fence_inside_list() {
        let input = "- Item\n```\ncode\n```";
        let output = preprocess_nested_blocks(input);
        assert!(output.contains("- Item\n    ```\n    code\n    ```"));
    }

    #[test]
    fn test_plugin_inside_list() {
        let input = "- Item\n@note(info){text}";
        let output = preprocess_nested_blocks(input);
        assert!(output.contains("- Item\n    @note(info){text}"));
    }

    #[test]
    fn test_blockquote_inside_list() {
        let input = "- Item\n> Quote\n> Next";
        let output = preprocess_nested_blocks(input);
        assert!(output.contains("- Item\n    > Quote\n    > Next"));
    }

    #[test]
    fn test_nested_list_not_modified() {
        let input = "- Item\n  - Nested\n  - Nested 2";
        let output = preprocess_nested_blocks(input);
        assert_eq!(output, input);
    }
}
