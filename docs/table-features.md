# テーブル機能

**最終更新**: 2026年5月18日

Universal Markdown のテーブル関連機能です。

## 対応テーブル

- GFM テーブル → `<table class="umd-list-table">`（縦線なし。行の区切り線のみ）
- UMD テーブル（セル連結対応、PukiWiki風）→ `<table class="umd-table">`（縦線あり。フルグリッド）

デザインはUMDリファレンスCSS（余白・罫線・見出しの太さ）で定義しており、
外部CSSフレームワークの `table-striped`/`table-hover`/`table-dark` のような装飾バリエーションは
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
- 色・サイズ: `COLOR(fg,bg)`, `SIZE(value)`

配置プレフィックスは物理方向（`LEFT`/`RIGHT`/`TOP`/`BOTTOM`）ではなく論理方向の名称です。
`block_decorations.rs`の段落装飾（[umd-extensions.md](umd-extensions.md)参照）と同じ命名規則で、
`scss/utilities/text.scss`の`umd-start`/`umd-end`/`umd-v-start`等のクラスにそのまま対応します。

`COLOR()`/`SIZE()`は値のパレット・クラス名規則をインラインプラグイン`&color()`/`&size()`と共有します（`decoration_values.rs`）。

- `COLOR(fg,bg)`: 16色パレット名（`blue`/`indigo`/`violet`/`purple`/`pink`/`red`/`orange`/`amber`/`yellow`/`lime`/`green`/`teal`/`cyan`/`brown`/`gray`/`pewter`）→`umd-color-*`/`umd-bg-*`クラス。`ParserOptions.allow_hex_colors`が有効な場合は`#rgb`/`#rrggbb`も受け付け、インライン`style`に変換
- `SIZE(value)`: キーワード（`xs`/`sm`/`lg`/`xl`）→`umd-text-size-*`クラス。`ParserOptions.allow_custom_font_size`が有効な場合は任意のrem/px/数値も受け付け、インライン`style`に変換
- 適用先は`&color()`/`&size()`が生成する`<span>`ではなく、セル自体（`<td>`/`<th>`）。背景色・文字色はセル単位の属性であり、文中の一部だけを装飾する`&color()`とは役割が異なるため。セル内容中に`項目&color(red){New!};`のようにネストした`&color()`で部分的に上書きすることも可能

## `@table` プラグインは廃止

`@table(options){{ ... }}`（テーブルへの外部CSSバリエーション適用）は削除されました。
UMDリファレンスCSSへ移行したことで、外部フレームワーク依存のテーブルバリエーション適用は
このライブラリの責務ではなくなったためです。`@table`/`:::table`は未知のプラグイン名として
扱われ、他の未対応プラグインと同様に汎用の`<template class="umd-plugin umd-plugin-table">`
にフォールバックします（詳細は[plugin-system.md](plugin-system.md)）。

## 実装の主担当

- `src/extensions/table/umd/`（UMDテーブルのパース・セル連結・セル装飾）
- `src/extensions/table/gfm.rs`（GFMテーブルの既定クラス・UMDテーブル
  マーカー復元）
- `src/extensions/alignment.rs`（テーブルセルの配置プレフィックス、
  GFM/UMD双方で共有）
- `src/extensions/conflict_resolver.rs`

## 主なテスト

- `tests/bootstrap_integration.rs`
- `examples/test_table_colspan.rs`
- `examples/test_comrak_table.rs`
- `examples/test_table_comparison.rs`
