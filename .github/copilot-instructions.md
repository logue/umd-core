# Copilot Instructions for umd-core

**最終更新**: 2026年3月3日

AI駆動開発のためのクイックリファレンス。詳細は [docs/architecture.md](../docs/architecture.md) を参照してください。

## 最初に読むべき内容

**パイプライン**（src/lib.rs:parse_with_frontmatter_opts）:

```
frontmatter抽出 → 前処理 → 競合保護 → サニタイズ → comrakパース
→ 拡張機能 → フットノート抽出 → 後処理 → HTML出力
```

**重要**: パイプラインの処理順序は変更するべからず。詳細は `docs/architecture.md` の「パイプライン処理順序」セクション参照。

## 何をどこで変更するか（クイックリファレンス）

| 対象機能                               | ファイル                               | テスト                           |
| -------------------------------------- | -------------------------------------- | -------------------------------- |
| 構文競合、UMD仕様                      | `src/extensions/conflict_resolver.rs`  | `tests/conflict_resolution.rs`   |
| インライン装飾 (`&color`, `&ruby` 等)  | `src/extensions/inline_decorations.rs` | `tests/bootstrap_integration.rs` |
| ブロック装飾 (`COLOR()`, `CENTER:` 等) | `src/extensions/block_decorations.rs`  | `tests/bootstrap_integration.rs` |
| プラグイン (`&fn()`, `@fn()`)          | `src/extensions/plugins.rs`            | `tests/ *`                       |
| コードブロック                         | `src/extensions/code_block.rs`         | `tests/bootstrap_integration.rs` |
| テーブル拡張                           | `src/extensions/table/umd/*`           | `examples/test_table_*.rs`       |
| メディア自動検出                       | `src/extensions/media.rs`              | `tests/bootstrap_integration.rs` |

詳細は `docs/architecture.md` の「何をどこで変更するか」セクション参照。

## プロジェクト規約（簡潔版）

- **パイプライン優先**: ローカル「整理」より処理順序の安定性を重視
- **コード保護**: 新規正規表現は `protect_code_sections` パターンを回避しない
- **UMD構文**:
  - ブロック引用: `> ... <` → `<blockquote class="umd-blockquote">`
  - 下線: `__text__` → `<u>`
  - プラグイン出力: `<template class="umd-plugin">` (バックエンド側で実行)
  - Base URL: `ParserOptions.base_url` で opt-in

詳細は `docs/architecture.md` の「プロジェクト固有の規約」セクション参照。

## ワークフロー

### ドキュメント駆動

1. `PLAN.md` を読んでタスク理解
2. 新ルールは `docs/` に文書化（コード前に）
3. **完了定義**: コード + テストパス + 仕様更新

### ビルド・テスト

```bash
# ローカルCI実行
cargo build --verbose && cargo test --verbose

# 高速検証
cargo test --test conflict_resolution
cargo test --test bootstrap_integration

# WASMビルド
./build.sh [dev|release]

# デモ実行
cargo run --example test_plugin_extended
```

詳細は `docs/architecture.md` の「ビルド・テスト・デバッグワークフロー」セクション参照。

## crates.io公開ドキュメント運用

Rust crate の公開関連は以下を一次情報として扱う：

- `PUBLISHING.md` - crates.io 公開手順（事前確認、package確認、publish、公開後検証）
- `RELEASE.md` - リリース運用（SemVer、タグ、リリースノート）
- `CHANGELOG.md` - 変更履歴（Keep a Changelog 形式）
- `SECURITY.md` - 脆弱性報告（GitHub Security Advisory）

運用ルール：

- 公開前に `cargo build --verbose && cargo test --verbose` を必須実行
- 公開前に `cargo package --list` で同梱物を確認
- 公開時は Rust crate のみを対象とし、`pkg/` の npm 公開手順は本スコープ外
- `README.md` は利用者向け概要、公開運用詳細は上記4文書へ分離する

## ドキュメント体系

- **[docs/architecture.md](../docs/architecture.md)** - システムアーキテクチャ、処理フロー、コンポーネント詳細、開発者ガイド
- **[docs/implemented-features.md](../docs/implemented-features.md)** - 実装済み機能のリファレンス
- **[docs/planned-features.md](../docs/planned-features.md)** - 実装予定機能の仕様
- **[PLAN.md](../PLAN.md)** - 未実装機能と実装計画
- **[README.md](../README.md)** - プロジェクト概要とユーザー向けドキュメント
- **[PUBLISHING.md](../PUBLISHING.md)** - crates.io 公開手順
- **[RELEASE.md](../RELEASE.md)** - リリース運用手順
- **[CHANGELOG.md](../CHANGELOG.md)** - 変更履歴
- **[SECURITY.md](../SECURITY.md)** - セキュリティポリシー

## 複数AI対応

このプロジェクトは複数のAIエージェント（Gemini, Grok等）での開発を想定しています。以下を徹底してください：

- 明確で冗長性のあるドキュメント
- 仕様書とコードの一貫性維持
- 実装完了は「テストパス」と「仕様更新」の両方で定義
