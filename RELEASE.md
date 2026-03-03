# Release Process

This document defines maintainers' release workflow for `umd`.

## Versioning policy

This project follows Semantic Versioning.

- Patch: bug fixes, no breaking API changes
- Minor: backward-compatible features
- Major: breaking API/behavior changes

## Release checklist

1. Update `Cargo.toml` version.
2. Update `CHANGELOG.md` for the target version.
3. Run validation:

```bash
cargo build --verbose
cargo test --verbose
```

4. Verify package contents:

```bash
cargo package --list
```

5. Publish according to `PUBLISHING.md`.
6. Create git tag: `vX.Y.Z`.
7. Create GitHub Release with summary from `CHANGELOG.md`.
8. Update or add a versioned release note file (e.g. `RELEASE_NOTES_v0.1.0.md`) and paste it into the GitHub Release body.

## Release notes format

Use concise, user-facing bullets grouped by:

- Added
- Changed
- Fixed
- Removed

## Rollback policy

Published crates cannot be deleted. If a release is incorrect:

1. Publish a corrected patch version.
2. Document the issue and fix in `CHANGELOG.md`.
3. Communicate recommended upgrade path in GitHub Release notes.
