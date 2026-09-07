# テーブル機能

**最終更新**: 2026年5月18日

Universal Markdown のテーブル関連機能です。

## 対応テーブル

- GFM テーブル → `<table class="umd-list-table">`（縦線なし。行の区切り線のみ）
- UMD テーブル（セル連結対応、PukiWiki風）→ `<table class="umd-table">`（縦線あり。フルグリッド）

デザインはBootstrapのテーブル（余白・罫線・見出しの太さ）に準拠しますが、
`table-striped`/`table-hover`/`table-dark`のような装飾バリエーションは
サポートしません。縦線の有無はどちらの構文で書いたかで決まる固定の性質であり、
オプションで切り替えるものではないため、`table-bordered`/`table-borderless`も
非対応です。スタイル定義は[`scss/components/table.scss`](../scss/components/table.scss)参照。

## UMDテーブル拡張

### セル連結

- 横連結: `|>` -> `colspan`
- 縦連結: `|^` -> `rowspan`

### セル装飾

セル内容に装飾プレフィックスを適用できます。

- インライン方向の配置: `START:`, `CENTER:`, `END:`, `JUSTIFY:`
- ブロック方向の配置: `V-START:`, `V-CENTER:`, `V-END:`, `BASELINE:`
- 色・サイズ: `COLOR(...)`, `SIZE(...)`

配置プレフィックスは物理方向（`LEFT`/`RIGHT`/`TOP`/`BOTTOM`）ではなく論理方向の名称です。
`block_decorations.rs`の段落装飾（[umd-extensions.md](umd-extensions.md)参照）と同じ命名規則で、
`scss/utilities/text.scss`の`umd-start`/`umd-end`/`umd-v-start`等のクラスにそのまま対応します。

## `@table` プラグインは廃止

`@table(options){{ ... }}`（テーブルへのBootstrapバリエーション適用）は削除されました。
Bootstrapに依存しない設計に移行したことで、テーブルの見た目のバリエーション適用は
このライブラリの責務ではなくなったためです。`@table`/`:::table`は未知のプラグイン名として
扱われ、他の未対応プラグインと同様に汎用の`<template class="umd-plugin umd-plugin-table">`
にフォールバックします（詳細は[plugin-system.md](plugin-system.md)）。

## 実装の主担当

- `src/extensions/table/umd/`
- `src/extensions/conflict_resolver/table/`
- `src/extensions/conflict_resolver.rs`

## 主なテスト

- `tests/bootstrap_integration.rs`
- `examples/test_table_colspan.rs`
- `examples/test_comrak_table.rs`
- `examples/test_table_comparison.rs`
