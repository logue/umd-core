# テーブル機能

**最終更新**: 2026年5月18日

Universal Markdown のテーブル関連機能です。

## 対応テーブル

- GFM テーブル
- UMD テーブル（セル連結対応）

## UMDテーブル拡張

### セル連結

- 横連結: `|>` -> `colspan`
- 縦連結: `|^` -> `rowspan`

### セル装飾

セル内容に装飾プレフィックスを適用できます。

- 配置: `LEFT:`, `CENTER:`, `RIGHT:`
- 縦位置: `TOP:`, `MIDDLE:`, `BOTTOM:`
- 色・サイズ: `COLOR(...)`, `SIZE(...)`

## `@table` プラグイン

`@table(options){{ ... }}` で最初のテーブルにのみオプションを適用。

- `striped` -> `table-striped`
- `hover` -> `table-hover`
- `dark` -> `table-dark`
- `bordered` -> `table-bordered`
- `borderless` -> `table-borderless`
- `sm` -> `table-sm`
- `responsive` -> `<div class="table-responsive">` でラップ

## 実装の主担当

- `src/extensions/table/umd/`
- `src/extensions/conflict_resolver/table/`
- `src/extensions/conflict_resolver.rs`

## 主なテスト

- `tests/bootstrap_integration.rs`
- `examples/test_table_colspan.rs`
- `examples/test_comrak_table.rs`
- `examples/test_table_comparison.rs`
