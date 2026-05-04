# Security Policy

## Reporting a Vulnerability

Please report security issues through GitHub Security Advisory:

- <https://github.com/logue/umd-core/security/advisories/new>

Do not open public issues for unpatched vulnerabilities.

## What to include

Please include:

- Affected version(s)
- Reproduction steps or proof of concept
- Impact assessment
- Suggested mitigation (if available)

## Response process

Maintainers will:

1. Acknowledge receipt.
2. Assess severity and impact.
3. Prepare and release a fix.
4. Publish coordinated disclosure details after remediation.

## Scope

This policy covers the Rust crate in this repository. External integrations and downstream deployments are handled by their respective maintainers.

## Security controls (current behavior)

- Input HTML is always escaped before parsing.
- Dangerous URL schemes are blocked: `javascript:`, `data:`, `vbscript:`, `file:`.
- Disallowed invisible blank-like characters are removed from text and URL inputs:
  - `U+200B`, `U+200C`, `U+200D`, `U+FEFF`, `U+3164`
- Allowed blank characters are only:
  - `U+0020` (half-width space)
  - `U+3000` (full-width space)
- Homograph-risk mitigation for external links:
  - For `http/https` links with non-ASCII hostnames or `xn--` punycode labels, UMD adds a visual warning marker (`class="umd-idn-warning-link"`, `data-idn-warning="true"`) and a warning icon element.
  - This is a visual warning and does not block the link.
