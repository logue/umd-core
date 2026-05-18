# UMD拡張構文

**最終更新**: 2026年5月18日

Universal Markdown 独自の構文拡張をまとめた仕様です。

## 主要機能

- UMD 強調構文
  - `''太字''` -> `<b>`
  - `'''斜体'''` -> `<i>`
  - `__下線__` -> `<u>`
- UMD 取り消し線
  - `%%...%%` -> `<s>`
- Spoiler
  - `||...||`
  - `&spoiler{...};`
- 定義リスト
  - `:term|definition`
- UMD ブロック引用
  - `> ... <`

## ブロック装飾プレフィックス

行頭プレフィックスで段落/ブロックを装飾します。

- 配置: `LEFT:`, `CENTER:`, `RIGHT:`, `JUSTIFY:`, `TRUNCATE:`
- 色: `COLOR(...)`
- サイズ: `SIZE(...)`
- 複合指定: `SIZE(...): COLOR(...): CENTER: ...`

## インライン装飾関数

- 見た目: `&color`, `&size`, `&badge`
- セマンティック: `&abbr`, `&ruby`, `&time`, `&kbd`, `&cite` など
- 改行/折返し: `&br;`, `&wbr;`

## ネスト深度制限

インライン装飾関数の再帰展開には上限があります。

- 設定: `ParserOptions.max_inline_nesting`
- 既定値: `Some(5)`
- 超過時: 該当部分をエラー表示クラスで無効化

## Step 6: 高度なUMD機能

- 数式
  - `&math(...)`
  - `@math(...)`
- Popover
  - `&popover(...)`
  - `@popover(...)`
- ネストブロック補正（リスト直下のブロック要素）
- タスクリスト拡張（`[-]` の indeterminate）
- カスタムリンク属性（`{#id .class}`）

## 実装の主担当

- `src/extensions/inline_decorations.rs`
- `src/extensions/block_decorations.rs`
- `src/extensions/conflict_resolver.rs`
- `src/extensions/nested_blocks.rs`

## 主なテスト

- `tests/bootstrap_integration.rs`
- `tests/conflict_resolution.rs`
- `tests/test_semantic_integration.rs`
- `examples/test_bootstrap_integration.rs`
