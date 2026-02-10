//! Example: Media embedding with auto-detection
//!
//! This example demonstrates the media file auto-detection feature
//! that converts image syntax to appropriate HTML5 media tags and
//! generates download links for non-media files.

use universal_markdown::parse;

fn main() {
    println!("=== Media Embedding Examples ===\n");

    // Video example
    let video_input = "![Product demo](https://example.com/video.mp4 \"Our new product\")";
    let video_html = parse(video_input);
    println!("Video Input:\n{}\n", video_input);
    println!("Video Output:\n{}\n", video_html);

    // Audio example
    let audio_input = "![Background music](https://example.com/audio.mp3 \"Theme song\")";
    let audio_html = parse(audio_input);
    println!("Audio Input:\n{}\n", audio_input);
    println!("Audio Output:\n{}\n", audio_html);

    // Image example with JPEG XL
    let image_input = "![Modern image](image.jxl \"JPEG XL format\")";
    let image_html = parse(image_input);
    println!("Image Input:\n{}\n", image_input);
    println!("Image Output:\n{}\n", image_html);

    // Downloadable file examples
    println!("=== Downloadable File Examples ===\n");

    let pdf_input = "![User Manual](manual.pdf \"Complete documentation\")";
    let pdf_html = parse(pdf_input);
    println!("PDF Input:\n{}\n", pdf_input);
    println!("PDF Output:\n{}\n", pdf_html);

    let zip_input = "![Source Code](project.zip)";
    let zip_html = parse(zip_input);
    println!("ZIP Input:\n{}\n", zip_input);
    println!("ZIP Output:\n{}\n", zip_html);

    let docx_input = "![Report](annual-report.docx \"Annual Report 2026\")";
    let docx_html = parse(docx_input);
    println!("DOCX Input:\n{}\n", docx_input);
    println!("DOCX Output:\n{}\n", docx_html);

    // Multiple media in one document
    let mixed_input = r#"# Project Showcase

Here's our product demo:

![Product Demo](demo.mp4 "Watch our amazing product in action")

Listen to our theme song:

![Theme Song](theme.mp3 "Composed by our team")

And here's our logo:

![Company Logo](logo.png "High-res logo")

Download our documentation:

![User Guide](user-guide.pdf "Complete user guide")

Get the source code:

![Source Archive](source.zip "Project source code")
"#;
    let mixed_html = parse(mixed_input);
    println!("Mixed Content Input:\n{}\n", mixed_input);
    println!("Mixed Content Output:\n{}\n", mixed_html);
}
