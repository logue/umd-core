# ドキュメント索引

**最終更新**: 2026年9月9日

Universal Markdown (UMD) の主要ドキュメントへの入口です。

## ディレクトリ構成

```
docs/
  spec/        ← UMD言語仕様（言語中立・外部参照用）
  *.md         ← Rust実装固有のドキュメント
```

`spec/` 以下はVSCode拡張・ドキュメントサイト・サードパーティ実装者が参照する仕様書です。  
実装固有の内部設計・テスト・ロードマップは `docs/` 直下に置きます。

---

## 目的別ガイド

### UMD言語仕様（`spec/`）

- **全体像を把握したい** → [spec/README.md](spec/README.md)
- **基本Markdown機能を確認したい** → [spec/basic-markdown-features.md](spec/basic-markdown-features.md)
- **UMD拡張構文を確認したい** → [spec/umd-extensions.md](spec/umd-extensions.md)
- **コメント構文の仕様を確認したい** → [spec/comment-syntax.md](spec/comment-syntax.md)
- **メディアタグ変換の仕様を確認したい** → [spec/media-tags.md](spec/media-tags.md)
- **プラグイン仕様を確認したい** → [spec/plugin-system.md](spec/plugin-system.md)
- **ブロック型プラグインを確認したい** → [spec/block-plugins.md](spec/block-plugins.md)
- **インライン型プラグインの一覧を確認したい** → [spec/inline-plugins.md](spec/inline-plugins.md)
- **テーブル機能を確認したい** → [spec/table-features.md](spec/table-features.md)
- **コードブロック構文を確認したい** → [spec/code-block-specification.md](spec/code-block-specification.md)
- **ファイル形式仕様を確認したい** → [spec/file-format.md](spec/file-format.md)
- **DPUB-ARIA対応仕様を確認したい** → [spec/dpub-aria.md](spec/dpub-aria.md)
- **テンプレートエンジン仕様の詳細を確認したい** → [spec/template-engine-spec.md](spec/template-engine-spec.md)
- **リファレンスCSSクラスを確認したい** → [spec/reference-css.md](spec/reference-css.md)

### Rust実装ドキュメント

- **実装アーキテクチャを確認したい** → [architecture.md](architecture.md)
- **実装済み仕様を確認したい** → [implemented-features.md](implemented-features.md)
- **コードブロック実装詳細を確認したい** → [code-block-extensions.md](code-block-extensions.md)
- **実行時機能（フロントマター・脚注）を確認したい** → [runtime-features.md](runtime-features.md)
- **セキュリティ仕様を確認したい** → [security-features.md](security-features.md)
- **未実装/提案仕様を確認したい** → [planned-features.md](planned-features.md)

### プロジェクト管理

- **実装計画・進捗を確認したい** → [PLAN.md](../PLAN.md)
- **WASMビルド手順を確認したい** → [WASM_BUILD.md](../WASM_BUILD.md)
- **crate公開手順を確認したい** → [PUBLISHING.md](../PUBLISHING.md)
- **リリース運用を確認したい** → [RELEASE.md](../RELEASE.md)
- **変更履歴を確認したい** → [CHANGELOG.md](../CHANGELOG.md)
- **脆弱性報告手順を確認したい** → [SECURITY.md](../SECURITY.md)

---

## 優先参照順（開発時）

1. [architecture.md](architecture.md)（処理順序・設計原則）
2. [implemented-features.md](implemented-features.md)（現行仕様）
3. [planned-features.md](planned-features.md)（将来仕様）

## メンテナンス方針

- 処理順序に関する記述は `src/lib.rs::parse_with_frontmatter_opts` を正とする
- 同一内容の重複記載は避け、詳細は上記の一次ドキュメントへ集約する
- `spec/` 以下は実装非依存で保つ。Rust固有の詳細は `docs/` 直下へ
