# Changelog

All notable changes to this project are documented in this file.

The format is based on Keep a Changelog.

## [Unreleased]

### Added

- Publishing and maintenance documentation: `PUBLISHING.md`, `RELEASE.md`, `SECURITY.md`.

### Fixed

- **Inline/block plugin nesting corrupted output** (`&function(args){content};` /
  `@function(args){{content}}`). `protect_inline_plugins`
  (`src/extensions/plugins/inline.rs`) and `protect_block_plugins`
  (`src/extensions/plugins/block.rs`) previously turned plugin calls into markers
  using whole-text regex passes that matched greedily up to the *first* closing
  delimiter. In practice this meant: a standard plugin nested more than one level
  inside another's `content` (`&color(blue){outer &abbr(t){d}; end};`) corrupted
  the marker stream, leaking fragments of the inner call as literal text; a
  plugin's `args` containing any unescaped `)` — most commonly a Markdown link,
  e.g. `&abbr([HTML](exp.html)){Hyper Text Markup Language};` — failed to match
  at all; and two sibling or nested block plugins sharing a multiline
  `{{...}}` body could have the first plugin's lazy content match swallow into
  its neighbor's closing `}}`, corrupting both. Both functions are now a single
  left-to-right scan (`parse_inline_plugin_at` / `parse_block_plugin_at`) that
  tracks delimiter nesting depth via new shared helpers
  `scan_balanced`/`scan_balanced_double` (`src/extensions/plugins/mod.rs`),
  so nested calls and parenthesized args resolve correctly at arbitrary depth.
  Marker formats and restore-side behavior (`restore_markers`, the `expand()`
  second-pass sweep, `max_inline_nesting`) are unchanged. Note that a plugin's
  `args`/`content` is still captured as opaque text rather than reparsed for
  nested plugins or Markdown — see
  [docs/known-issues.md](docs/known-issues.md) for what that does and
  doesn't cover after this fix.

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
