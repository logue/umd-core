# Publishing to crates.io

This document defines the standard release path for publishing the `umd` crate to crates.io.

## Scope

- Target: Rust crate publication to crates.io
- Out of scope: npm publication for `pkg/`

## Pre-publish checklist

Run these commands from repository root:

```bash
cargo build --verbose
cargo test --verbose
```

Confirm package metadata in `Cargo.toml`:

- `name`, `version`, `license`, `description`
- `repository`, `homepage`, `documentation`
- `keywords`, `categories`
- `readme`

## Verify packaged files

Inspect what will be uploaded:

```bash
cargo package --list
```

Ensure only expected files are included.

## Publish

Authenticate once (if needed):

```bash
cargo login <CRATES_IO_TOKEN>
```

Publish:

```bash
cargo publish
```

## Post-publish validation

1. Confirm crate page is updated on crates.io.
2. Confirm documentation builds on docs.rs.
3. Confirm `README.md` examples still match latest public API.

## Common failure cases

- Version already exists: bump `version` in `Cargo.toml` and retry.
- Missing metadata warning: update metadata fields before publish.
- Unexpected packaged files: adjust `.gitignore` or package file layout and re-check with `cargo package --list`.
