//! Media file auto-detection and HTML generation
//!
//! This module provides functionality to detect media files by extension
//! and generate appropriate HTML5 media tags (video, audio, picture).

use std::path::Path;

/// Media type detected from file extension
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MediaType {
    Video,
    Audio,
    Image,
}

/// Detect media type from URL
///
/// # Arguments
///
/// * `url` - The URL to analyze (may include query parameters)
///
/// # Returns
///
/// `Some(MediaType)` if a known extension is found, `None` otherwise
///
/// # Examples
///
/// ```
/// use universal_markdown::extensions::media::detect_media_type;
/// use universal_markdown::extensions::media::MediaType;
///
/// assert_eq!(detect_media_type("video.mp4"), Some(MediaType::Video));
/// assert_eq!(detect_media_type("audio.mp3"), Some(MediaType::Audio));
/// assert_eq!(detect_media_type("image.png"), Some(MediaType::Image));
/// assert_eq!(detect_media_type("file.unknown"), None);
/// ```
pub fn detect_media_type(url: &str) -> Option<MediaType> {
    // Extract path without query parameters or fragments
    let path = url.split('?').next()?.split('#').next()?;

    // Get extension
    let ext = Path::new(path).extension()?.to_str()?.to_lowercase();

    match ext.as_str() {
        // Video extensions
        "mp4" | "webm" | "ogv" | "mov" | "avi" | "mkv" | "m4v" => Some(MediaType::Video),
        // Audio extensions
        "mp3" | "wav" | "ogg" | "oga" | "m4a" | "aac" | "flac" | "opus" | "weba" => {
            Some(MediaType::Audio)
        }
        // Image extensions
        "jpg" | "jpeg" | "png" | "gif" | "svg" | "webp" | "avif" | "bmp" | "ico" | "jxl" => {
            Some(MediaType::Image)
        }
        _ => None,
    }
}

/// Get MIME type for a file extension
///
/// # Arguments
///
/// * `url` - The URL to analyze
/// * `media_type` - The detected media type
///
/// # Returns
///
/// MIME type string
fn get_mime_type(url: &str, media_type: &MediaType) -> String {
    let path = url
        .split('?')
        .next()
        .unwrap_or(url)
        .split('#')
        .next()
        .unwrap_or(url);
    let ext = Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    match media_type {
        MediaType::Video => match ext.as_str() {
            "mp4" => "video/mp4",
            "webm" => "video/webm",
            "ogv" | "ogg" => "video/ogg",
            "mov" => "video/quicktime",
            "avi" => "video/x-msvideo",
            "mkv" => "video/x-matroska",
            "m4v" => "video/x-m4v",
            _ => "video/mp4",
        },
        MediaType::Audio => match ext.as_str() {
            "mp3" => "audio/mpeg",
            "wav" => "audio/wav",
            "ogg" | "oga" => "audio/ogg",
            "m4a" => "audio/mp4",
            "aac" => "audio/aac",
            "flac" => "audio/flac",
            "opus" => "audio/opus",
            "weba" => "audio/webm",
            _ => "audio/mpeg",
        },
        MediaType::Image => match ext.as_str() {
            "jpg" | "jpeg" => "image/jpeg",
            "png" => "image/png",
            "gif" => "image/gif",
            "svg" => "image/svg+xml",
            "webp" => "image/webp",
            "avif" => "image/avif",
            "bmp" => "image/bmp",
            "ico" => "image/x-icon",
            "jxl" => "image/jxl",
            _ => "image/png",
        },
    }
    .to_string()
}

/// Generate HTML for media element
///
/// # Arguments
///
/// * `url` - The media URL
/// * `alt` - Alt text (used for track label in video, ignored in audio)
/// * `title` - Optional title attribute
/// * `media_type` - The type of media
///
/// # Returns
///
/// HTML string for the media element
///
/// # Examples
///
/// ```
/// use universal_markdown::extensions::media::{generate_media_html, MediaType};
///
/// let html = generate_media_html("video.mp4", "Demo", Some("Product demo"), &MediaType::Video);
/// assert!(html.contains("<video"));
/// assert!(html.contains("controls"));
/// ```
pub fn generate_media_html(
    url: &str,
    alt: &str,
    title: Option<&str>,
    media_type: &MediaType,
) -> String {
    let mime_type = get_mime_type(url, media_type);
    let title_attr = title
        .map(|t| format!(" title=\"{}\"", escape_html(t)))
        .unwrap_or_default();

    match media_type {
        MediaType::Video => {
            let track_label = escape_html(alt);
            format!(
                "<video controls{}>\n  <source src=\"{}\" type=\"{}\" />\n  <track kind=\"captions\" label=\"{}\" />\n  お使いのブラウザは動画タグをサポートしていません。\n</video>",
                title_attr,
                escape_html(url),
                mime_type,
                track_label
            )
        }
        MediaType::Audio => {
            format!(
                "<audio controls{}>\n  <source src=\"{}\" type=\"{}\" />\n  お使いのブラウザは音声タグをサポートしていません。\n</audio>",
                title_attr,
                escape_html(url),
                mime_type
            )
        }
        MediaType::Image => {
            let img_title = title
                .map(|t| format!(" title=\"{}\"", escape_html(t)))
                .unwrap_or_default();
            format!(
                "<picture{}>\n  <source srcset=\"{}\" type=\"{}\" />\n  <img src=\"{}\" alt=\"{}\" loading=\"lazy\"{} />\n</picture>",
                title_attr,
                escape_html(url),
                mime_type,
                escape_html(url),
                escape_html(alt),
                img_title
            )
        }
    }
}

/// Escape HTML special characters
fn escape_html(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

/// Transform image tags to media tags based on file extension
///
/// This function processes HTML and converts `<img>` tags to appropriate
/// media tags (`<video>`, `<audio>`, or `<picture>`) based on the file extension.
///
/// # Arguments
///
/// * `html` - The HTML string to transform
///
/// # Returns
///
/// Transformed HTML with media tags
///
/// # Examples
///
/// ```
/// use universal_markdown::extensions::media::transform_images_to_media;
///
/// let html = r#"<img src="video.mp4" alt="Demo" />"#;
/// let result = transform_images_to_media(html);
/// assert!(result.contains("<video"));
/// ```
pub fn transform_images_to_media(html: &str) -> String {
    use regex::Regex;

    // Pattern to match <img> tags with src and alt attributes
    let img_re =
        Regex::new(r#"<img\s+src="([^"]+)"(?:\s+alt="([^"]*)")?(?:\s+title="([^"]*)")?\s*/>"#)
            .unwrap();

    img_re
        .replace_all(html, |caps: &regex::Captures| {
            let url = caps.get(1).map_or("", |m| m.as_str());
            let alt = caps.get(2).map_or("", |m| m.as_str());
            let title = caps.get(3).map(|m| m.as_str());

            // Detect media type and generate appropriate HTML
            if let Some(media_type) = detect_media_type(url) {
                generate_media_html(url, alt, title, &media_type)
            } else {
                // Not a recognized media file, wrap in <picture> tag anyway
                let title_attr = title
                    .map(|t| format!(" title=\"{}\"", t))
                    .unwrap_or_default();
                let img_title = title
                    .map(|t| format!(" title=\"{}\"", t))
                    .unwrap_or_default();
                format!(
                    "<picture{}>\n  <img src=\"{}\" alt=\"{}\" loading=\"lazy\"{} />\n</picture>",
                    title_attr, url, alt, img_title
                )
            }
        })
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_video() {
        assert_eq!(detect_media_type("video.mp4"), Some(MediaType::Video));
        assert_eq!(detect_media_type("video.webm"), Some(MediaType::Video));
        assert_eq!(detect_media_type("video.ogv"), Some(MediaType::Video));
        assert_eq!(detect_media_type("video.mov"), Some(MediaType::Video));
        assert_eq!(detect_media_type("VIDEO.MP4"), Some(MediaType::Video)); // Case insensitive
    }

    #[test]
    fn test_detect_audio() {
        assert_eq!(detect_media_type("audio.mp3"), Some(MediaType::Audio));
        assert_eq!(detect_media_type("audio.wav"), Some(MediaType::Audio));
        assert_eq!(detect_media_type("audio.ogg"), Some(MediaType::Audio));
        assert_eq!(detect_media_type("AUDIO.MP3"), Some(MediaType::Audio)); // Case insensitive
    }

    #[test]
    fn test_detect_image() {
        assert_eq!(detect_media_type("image.png"), Some(MediaType::Image));
        assert_eq!(detect_media_type("image.jpg"), Some(MediaType::Image));
        assert_eq!(detect_media_type("image.jpeg"), Some(MediaType::Image));
        assert_eq!(detect_media_type("image.gif"), Some(MediaType::Image));
        assert_eq!(detect_media_type("image.webp"), Some(MediaType::Image));
        assert_eq!(detect_media_type("image.avif"), Some(MediaType::Image));
        assert_eq!(detect_media_type("image.jxl"), Some(MediaType::Image)); // JPEG XL
    }

    #[test]
    fn test_detect_with_query_params() {
        assert_eq!(detect_media_type("video.mp4?v=123"), Some(MediaType::Video));
        assert_eq!(
            detect_media_type("image.png?size=large#anchor"),
            Some(MediaType::Image)
        );
    }

    #[test]
    fn test_detect_unknown() {
        assert_eq!(detect_media_type("file.txt"), None);
        assert_eq!(detect_media_type("file.doc"), None);
        assert_eq!(detect_media_type("noextension"), None);
    }

    #[test]
    fn test_generate_video_html() {
        let html = generate_media_html(
            "video.mp4",
            "Demo video",
            Some("Product demo"),
            &MediaType::Video,
        );
        assert!(html.contains("<video controls"));
        assert!(html.contains("title=\"Product demo\""));
        assert!(html.contains("src=\"video.mp4\""));
        assert!(html.contains("type=\"video/mp4\""));
        assert!(html.contains("<track kind=\"captions\" label=\"Demo video\""));
    }

    #[test]
    fn test_generate_audio_html() {
        let html = generate_media_html(
            "audio.mp3",
            "Background music",
            Some("Theme song"),
            &MediaType::Audio,
        );
        assert!(html.contains("<audio controls"));
        assert!(html.contains("title=\"Theme song\""));
        assert!(html.contains("src=\"audio.mp3\""));
        assert!(html.contains("type=\"audio/mpeg\""));
    }

    #[test]
    fn test_generate_image_html() {
        let html =
            generate_media_html("image.png", "Logo", Some("Company logo"), &MediaType::Image);
        assert!(html.contains("<picture"));
        assert!(html.contains("title=\"Company logo\""));
        assert!(html.contains("srcset=\"image.png\""));
        assert!(html.contains("type=\"image/png\""));
        assert!(html.contains("alt=\"Logo\""));
        assert!(html.contains("loading=\"lazy\""));
    }

    #[test]
    fn test_generate_without_title() {
        let html = generate_media_html("video.mp4", "Video", None, &MediaType::Video);
        assert!(!html.contains("title="));
        assert!(html.contains("<video controls>"));
    }

    #[test]
    fn test_html_escape() {
        let html = generate_media_html(
            "video.mp4?foo=bar&baz=qux",
            "Test <script>",
            Some("Title with \"quotes\""),
            &MediaType::Video,
        );
        assert!(html.contains("&amp;"));
        assert!(html.contains("&lt;"));
        assert!(html.contains("&quot;"));
    }
}
