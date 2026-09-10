# Changelog

All notable changes to this project are documented in this file.

The format is based on Keep a Changelog.

## [Unreleased]

### Added

- Publishing and maintenance documentation: `PUBLISHING.md`, `RELEASE.md`, `SECURITY.md`.

### Removed (BREAKING)

- Native Mermaid SSR (`mermaid-rs-renderer`) and Syntect-based syntax highlighting have been
  removed from `umd-core`. `render()`/`parse()` output no longer contains rendered Mermaid SVG
  (`<figure class="umd-code-block-mermaid umd-mermaid-diagram">...<svg>...`) or
  syntect-highlighted spans (`umd-syntect-highlight`, `data-highlighted="true"`). Fenced code
  blocks are now always emitted as plain `<figure class="umd-code-block"><pre><code
  class="language-*">...</code></pre></figure>`, regardless of language — including `mermaid`.
  Rendering and highlighting are host application (Layer 2/3) responsibilities; see
  [docs/code-block-extensions.md](docs/code-block-extensions.md). Rationale: both were
  native-only dependencies (they could never build for `wasm32`, so they were never actually
  part of the WASM binary despite earlier `PLAN.md` notes to the contrary) and represent
  domain-specific visual rendering that the 3-layer architecture places outside core
  ([docs/architecture.md](docs/architecture.md)).

## [0.1.0] - 2026-03-03

### Added

- Initial public release of Universal Markdown (`umd`).
- CommonMark and GFM-compatible parsing pipeline.
- UMD legacy syntax compatibility and extension modules.
- Native Mermaid SSR and Syntect highlighting support.
- WASM bindings for browser integration.
