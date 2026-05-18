# 実装済み機能リファレンス

**最終更新**: 2026年5月18日

このドキュメントは実装済み機能の索引です。詳細仕様はテーマ別ドキュメントを参照してください。

## 目次

- [テーマ別詳細ドキュメント](#テーマ別詳細ドキュメント)
- [基本Markdown機能](#基本markdown機能)
- [UMD拡張構文](#umd拡張構文)
- [コメント構文](#コメント構文)
- [メディアファイル自動検出](#メディアファイル自動検出)
- [Spoiler機能](#spoiler機能)
- [定義リスト](#定義リスト)
- [ブロック引用](#ブロック引用)
- [プラグインシステム](#プラグインシステム)
- [テーブル機能](#テーブル機能)
- [その他の機能](#その他の機能)
- [セキュリティ](#セキュリティ)
- [Step 6: 高度なUMD機能](#step-6-高度なumd機能)
- [テスト結果](#テスト結果)

## テーマ別詳細ドキュメント

- 基本Markdown機能: [basic-markdown-features.md](basic-markdown-features.md)
- UMD拡張構文: [umd-extensions.md](umd-extensions.md)
- コメント構文: [comment-syntax.md](comment-syntax.md)
- メディアタグ・自動検出: [media-tags.md](media-tags.md)
- プラグインシステム: [plugin-system.md](plugin-system.md)
- テーブル機能: [table-features.md](table-features.md)
- 実行時機能（フロントマター・脚注・出力）: [runtime-features.md](runtime-features.md)
- セキュリティ仕様: [security-features.md](security-features.md)

---

## 基本Markdown機能

実装済み。詳細は [basic-markdown-features.md](basic-markdown-features.md) を参照してください。

補足:

- コードブロック拡張の詳細は [code-block-extensions.md](code-block-extensions.md)

## UMD拡張構文

実装済み。詳細は [umd-extensions.md](umd-extensions.md) を参照してください。

## コメント構文

実装済み。詳細は [comment-syntax.md](comment-syntax.md) を参照してください。

## メディアファイル自動検出

実装済み。詳細は [media-tags.md](media-tags.md) を参照してください。

## Spoiler機能

実装済み。詳細は [umd-extensions.md](umd-extensions.md) を参照してください。

## 定義リスト

実装済み。詳細は [umd-extensions.md](umd-extensions.md) を参照してください。

## ブロック引用

実装済み。詳細は [umd-extensions.md](umd-extensions.md) を参照してください。

## プラグインシステム

実装済み。詳細は [plugin-system.md](plugin-system.md) を参照してください。

## テーブル機能

実装済み。詳細は [table-features.md](table-features.md) を参照してください。

## その他の機能

実装済み。詳細は [runtime-features.md](runtime-features.md) を参照してください。

## セキュリティ

実装済み。詳細は [security-features.md](security-features.md) を参照してください。

## Step 6: 高度なUMD機能

実装済み。詳細は [umd-extensions.md](umd-extensions.md) を参照してください。

## テスト結果

最新の検証はローカル CI 相当コマンドを参照してください。

```bash
cargo build --verbose && cargo test --verbose
```

高速確認:

```bash
cargo test --test conflict_resolution
cargo test --test bootstrap_integration
```
