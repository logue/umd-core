//! Fence info-string normalization: filename syntax (```lang:filename).

const CODEBLOCK_FILENAME_LANGLESS_MARKER: &str = "umd-nolang";
const CODEBLOCK_FILENAME_META_PREFIX: &str = "umd-filename:";

/// Normalize fenced code block info string for filename syntax.
///
/// Converts `lang:filename` to `lang umd-filename:filename` so comrak can emit
/// `data-meta` when `render.full_info_string = true`.
///
/// Also supports `:filename` by using an internal language marker (`umd-nolang`).
pub fn preprocess_code_block_filenames(input: &str) -> String {
    let ends_with_newline = input.ends_with('\n');
    let mut result = String::new();
    let mut in_code_block = false;
    let mut fence_char = '\0';
    let mut fence_len = 0usize;

    for line in input.lines() {
        let trimmed = line.trim_start();

        if !in_code_block {
            if let Some((prefix_len, current_fence_char, current_fence_len, info)) =
                parse_fence_open_line(trimmed)
            {
                in_code_block = true;
                fence_char = current_fence_char;
                fence_len = current_fence_len;

                let normalized_info = normalize_code_fence_info(info);
                let prefix = &line[..line.len() - trimmed.len()];
                let fence_marker = &trimmed[..prefix_len];

                result.push_str(prefix);
                result.push_str(fence_marker);
                if !normalized_info.is_empty() {
                    result.push(' ');
                    result.push_str(&normalized_info);
                }
                result.push('\n');
                continue;
            }
        } else if is_fence_close_line(trimmed, fence_char, fence_len) {
            in_code_block = false;
            fence_char = '\0';
            fence_len = 0;
            result.push_str(line);
            result.push('\n');
            continue;
        }

        result.push_str(line);
        result.push('\n');
    }

    if !ends_with_newline && result.ends_with('\n') {
        result.pop();
    }

    result
}

fn parse_fence_open_line(trimmed_line: &str) -> Option<(usize, char, usize, &str)> {
    let bytes = trimmed_line.as_bytes();
    if bytes.is_empty() {
        return None;
    }

    let first = bytes[0] as char;
    if first != '`' && first != '~' {
        return None;
    }

    let mut marker_len = 0usize;
    while marker_len < bytes.len() && (bytes[marker_len] as char) == first {
        marker_len += 1;
    }

    if marker_len < 3 {
        return None;
    }

    let info = trimmed_line[marker_len..].trim();
    Some((marker_len, first, marker_len, info))
}

fn is_fence_close_line(trimmed_line: &str, fence_char: char, fence_len: usize) -> bool {
    if trimmed_line.is_empty() || fence_len < 3 {
        return false;
    }

    let marker: String = std::iter::repeat(fence_char).take(fence_len).collect();
    if !trimmed_line.starts_with(&marker) {
        return false;
    }

    trimmed_line[fence_len..].trim().is_empty()
}

fn normalize_code_fence_info(info: &str) -> String {
    if info.is_empty() || info.contains(' ') {
        return info.to_string();
    }

    if let Some(filename) = info.strip_prefix(':') {
        if filename.is_empty() {
            return info.to_string();
        }
        return format!(
            "{} {}{}",
            CODEBLOCK_FILENAME_LANGLESS_MARKER, CODEBLOCK_FILENAME_META_PREFIX, filename
        );
    }

    if let Some((lang, filename)) = info.split_once(':') {
        if lang.is_empty() || filename.is_empty() {
            return info.to_string();
        }
        return format!("{} {}{}", lang, CODEBLOCK_FILENAME_META_PREFIX, filename);
    }

    info.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preprocess_code_block_filename_with_language() {
        let input = "```rust:src/main.rs\nfn main() {}\n```";
        let output = preprocess_code_block_filenames(input);
        assert!(output.contains("``` rust umd-filename:src/main.rs"));
    }

    #[test]
    fn test_preprocess_code_block_filename_without_language() {
        let input = "```:config.yml\nkey: value\n```";
        let output = preprocess_code_block_filenames(input);
        assert!(output.contains("``` umd-nolang umd-filename:config.yml"));
    }

    #[test]
    fn test_preprocess_code_block_filename_ignores_inside_block() {
        let input = "```txt\nrust:main.rs\n```";
        let output = preprocess_code_block_filenames(input);
        assert!(output.contains("rust:main.rs"));
    }
}
