# umd v0.1.0

Released: 2026-03-03

Initial public release of **Universal Markdown (UMD)**, a Rust-first Markdown parser with CommonMark/GFM compatibility, UMD legacy syntax support, semantic HTML generation, and Bootstrap-friendly output.

## Highlights

- CommonMark + GFM support (tables, strikethrough, task lists, footnotes)
- UMD syntax compatibility (legacy decorators, plugins, block quote format)
- Semantic inline and block extensions (`&badge`, `&ruby`, `COLOR()`, `CENTER:`, etc.)
- Media auto-detection from image syntax (image/video/audio/download)
- Mermaid server-side rendering on native targets
- Syntect-based syntax highlighting with frontend fallback
- Frontmatter extraction (YAML/TOML) and structured footnote output
- WASM support via `wasm-bindgen`

## Security

- HTML input sanitization is enabled by default
- Dangerous URL schemes are blocked (`javascript:`, `data:`, `vbscript:`, `file:`)
- Plugin output is emitted as safe `<template>` blocks for backend-side processing

## Compatibility

- Rust: 1.93.1+
- Edition: 2024
- Targets: native Rust library + `wasm32-unknown-unknown`

## Install

```toml
[dependencies]
umd = "0.1.0"
```

## Notes

- No known breaking changes (initial release).
- See `CHANGELOG.md` for change history and `SECURITY.md` for vulnerability reporting.
