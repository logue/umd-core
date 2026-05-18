# プラグインシステム

**最終更新**: 2026年5月18日

Universal Markdown のプラグイン構文と出力形式です。

## 構文

### インライン型

- `&function(arg1,arg2){content};`
- `&function(args);`
- `&function;`

### ブロック型

- `@function(args){{ ... }}`
- `@function(args){...}`
- `@function(args)`
- `@function()`

## 出力形式

プラグインは次の形式で出力されます。

- `<template class="umd-plugin umd-plugin-{name}">...</template>`
- 引数は `<data value="index">...</data>` で保持
- コンテンツはエスケープ済みテキストとして保持

バックエンド側（Nuxt/Laravel 等）で再パースして最終描画する設計です。

## 標準プラグイン

- `@detail(summary[, open])`
  - `<details><summary>...</summary>...</details>`
- `@clear()`
  - `<div class="clearfix"></div>`
- `@table(...)`
  - テーブルへの Bootstrap バリエーション適用（詳細は [table-features.md](table-features.md)）

## 実装の主担当

- `src/extensions/plugins.rs`
- `src/extensions/plugin_markers.rs`
- `src/extensions/conflict_resolver.rs`

## 主なテスト

- `tests/bootstrap_integration.rs`
- `tests/conflict_resolution.rs`
- `examples/test_plugin_extended.rs`
- `examples/test_plugin_table.rs`
