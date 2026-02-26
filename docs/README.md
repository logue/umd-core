# ドキュメント索引

**最終更新**: 2026年2月26日

Universal Markdown (UMD) の主要ドキュメントへの入口です。

## 目的別ガイド

- **まず全体像を把握したい** → [README.md](../README.md)
- **実装アーキテクチャを確認したい** → [architecture.md](architecture.md)
- **実装済み仕様を確認したい** → [implemented-features.md](implemented-features.md)
- **未実装/提案仕様を確認したい** → [planned-features.md](planned-features.md)
- **実装計画・進捗を確認したい** → [PLAN.md](../PLAN.md)
- **WASMビルド手順を確認したい** → [WASM_BUILD.md](../WASM_BUILD.md)

## 優先参照順（開発時）

1. [architecture.md](architecture.md)（処理順序・設計原則）
2. [implemented-features.md](implemented-features.md)（現行仕様）
3. [planned-features.md](planned-features.md)（将来仕様）

## メンテナンス方針

- 処理順序に関する記述は `src/lib.rs::parse_with_frontmatter_opts` を正とする
- 同一内容の重複記載は避け、詳細は上記の一次ドキュメントへ集約する
- 長期的に参照頻度が低い引き継ぎ資料は、最新仕様との差分が出る前提で扱う
