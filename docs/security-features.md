# セキュリティ仕様

**最終更新**: 2026年5月18日

Universal Markdown の入力保護と出力安全化に関する実装仕様です。

## HTML入力制限

- 直接 HTML 入力はエスケープ
- パーサー生成 HTML のみを出力に利用
- `unsafe` レンダリングは無効

## ASCII制御文字除去

競合解決後、非表示制御文字を除去します。

- 対象: `U+0000`-`U+001F`（TAB/LF/CR 除く）、`U+007F`
- フェンスコードブロックは保護

## URL Sanitization

危険スキームをブロック:

- `javascript:`
- `data:`
- `vbscript:`
- `file:`（既定でブロック）

検査前に不可視文字を除去し、スキーム偽装を防止します。

## IDN警告マーカー

外部リンクのホストが IDN / punycode の場合、警告クラスと属性を付与します。

- `umd-idn-warning-link`
- `data-idn-warning="true"`

## プラグインコンテンツの責任分界

プラグイン内容の最終サニタイズは、実行側（バックエンド実装）の責任です。

## 実装の主担当

- `src/sanitizer.rs`
- `src/extensions/conflict_resolver.rs`
- `src/lib.rs`

## 主なテスト

- `src/sanitizer.rs` 内テスト
- `tests/conflict_resolution.rs`
