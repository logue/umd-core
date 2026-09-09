# AI Coding Assistant Guidelines & Project Stack

This project is a mono-repo template combining a TypeScript frontend/base infrastructure with Rust-based WebAssembly (Wasm) modules. AI assistants must strictly adhere to the following stack definitions, architectural conventions, and agent-skill integrations.

## 1. Technology Stack

- **Frontend / Base**: TypeScript
- **Linter / Formatter (TS-ecosystem)**: Biome (Do NOT suggest or use Prettier or ESLint)
- **Wasm / Backend**: Rust
- **Wasm Bindings**: wasm-bindgen, serde
- **Type Generation**: tsify-next (For auto-generating TS types from Rust structs/enums)
- **Wasm Build Tool**: wasm-pack (Managed via package.json scripts)

---

## 2. Formatting & Linting Conventions

### [TypeScript / JavaScript / JSON]

- **Tool**: `biome` is used for both linting and formatting.
- **Workflow**: Automated via **Format on Save** in the editor. No Git hooks (e.g., husky, lint-staged) are used.
- **Strict Rule**: Never suggest adding Prettier configuration files or Prettier-related plugins.

### [Rust]

- **Tool**: `cargo fmt` (`rustfmt`) for formatting, `cargo clippy` for linting.
- **Workflow**: Automated via **Format on Save** in the editor.
- **Strict Rule**:
  - Never suggest Node.js-based Rust formatters (e.g., `prettier-plugin-rust`).
  - Rely purely on official Rust toolchains.
  - Follow default `rustfmt` rules (Indent: 4 spaces, Max line length: 100).

---

## 3. Rust <=> TypeScript Boundary (Wasm Safety)

To maintain absolute type safety across boundaries, never write manual `.d.ts` files for Wasm interfaces or use `any` in TypeScript. Always use **`tsify-next`** to sync data structures.

### Rust Implementation Rules

1. Derive `Tsify`, `Serialize`, and `Deserialize` for all shared `struct` and `enum` definitions.
2. Use `#[tsify(into_wasm_abi, from_wasm_abi)]` to allow the struct to be passed directly through Wasm functions.
3. Apply `#[serde(rename_all = "camelCase")]` to automatically bridge Rust's `snake_case` fields to TypeScript's `camelCase` conventions.

**Correct Example (Rust):**

```rust
use serde::{Deserialize, Serialize};
use tsify_next::Tsify;

#[derive(Tsify, Serialize, Deserialize)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct UserProfile {
    pub user_id: String,
    pub display_name: String,
    pub roles: Vec<String>,
}
```

### TypeScript Integration Rules

1. Consume the generated `*.d.ts` types output by the `wasm-pack` build process directly.
2. Treat generated interfaces as read-only; never modify them manually.

---

## 4. Architecture: Deferred Rendering Model

umd-core follows a **3-layer deferred rendering** design. The core principle is:

> **The core does not need to know what a plugin means.**

### Layer Responsibilities

| Layer | Owner | Responsibility |
|-------|-------|----------------|
| **Layer 1: umd-core (Rust/WASM)** | This repo | Parse CommonMark + UMD standard syntax. Convert plugin syntax into `<template>`/`<data>` placeholder tags. The core has no knowledge of plugin semantics. |
| **Layer 2: Host application** | Consumer | Interpret `<template>`/`<data>` placeholders and expand them into actual HTML/content. Implement domain-specific plugins here. |
| **Layer 3: Frontend** | Consumer | Apply CSS classes (`umd-*`), handle client-side interaction and rendering. |

### Decision Criterion: What Belongs in Core

**Include in core**: Syntax that conforms to the CommonMark spec, or inline/block syntax that UMD standardizes across all use cases.

**Host/frontend responsibility**: Domain-specific processing, concrete interpretation of plugin semantics, visual presentation details.

When in doubt: if the meaning of a construct depends on *who is using UMD*, it belongs outside the core.

### `<template>`/`<data>` Placeholders

Plugin syntax that umd-core cannot resolve at parse time is emitted as a `<template>` or `<data>` tag. This ensures:

- Pages do not break if the host has not implemented the plugin — the tag is silently ignored by browsers.
- Host-specific rendering is achieved without modifying core.
- Frontend-only plugins are also possible.

```html
<!-- Input: &myplugin(arg){content} -->
<!-- umd-core output: -->
<template data-plugin="myplugin" data-arg="arg">content</template>
```

### Contrast with PukiWiki

PukiWiki plugins generate HTML directly by inserting PHP code into the core. umd-core inverts this: plugins are pushed *outside* the core, and communication happens through semantically neutral placeholder tags. The host decides how (and whether) to render them.
