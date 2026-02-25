# アーキテクチャドキュメント

**最終更新**: 2026年2月25日

Universal Markdownの技術アーキテクチャとシステム設計を記載しています。

## 目次

- [システム概要](#システム概要)
- [処理フロー](#処理フロー)
- [主要コンポーネント](#主要コンポーネント)
- [依存クレート](#依存クレート)
- [ディレクトリ構造](#ディレクトリ構造)
- [開発者向けガイド](#開発者向けガイド)
- [構文優先順位ポリシー](#構文優先順位ポリシー)
- [セキュリティ方針](#セキュリティ方針)

---

## システム概要

Universal Markdownは、CommonMark準拠のMarkdownパーサーをベースに、UMD（Universal Markdown）独自構文、Bootstrap 5統合、セマンティックHTML生成を実現するRust製パーサーライブラリです。

### 設計方針

1. **CommonMark準拠**: 75%以上のCommonMark仕様テストをパス
2. **セキュリティ優先**: HTML直接入力を禁止、全てのユーザー入力をエスケープ
3. **拡張性**: プラグインシステムによる機能拡張が可能
4. **Bootstrap統合**: Bootstrap 5のユーティリティクラスを自動生成
5. **セマンティックHTML**: アクセシビリティとSEOを考慮したHTML出力

### 技術スタック

- **言語**: Rust 1.93.1 (Edition 2024)
- **ベースパーサー**: comrak 0.50.0 (GFM準拠ASTベース)
- **HTML生成**: maud 0.27.0 (型安全)
- **HTML安全化**: ammonia 4.1.2
- **正規表現**: regex 1.12.2
- **WASM対応**: wasm-bindgen 0.2.108

---

## 処理フロー

```
Input Text
    ↓
[HTML Sanitizer] - HTMLエスケープ、エンティティ保持
    ↓
[Conflict Resolver] - UMD構文をマーカーで保護
    ↓
[Frontmatter Extractor] - YAMLFrontmatterを抽出・除去
    ↓
[comrak Parser] - Markdown → AST構築
    ↓
[UMD Extensions] - UMD独自ノード追加・変換
    ↓
[HTML Renderer] - AST → HTML変換
    ↓
[Plugin Processor] - プラグインを<template>タグに変換
    ↓
[Post Processor] - マーカーをHTMLに復元
    ↓
Output HTML + Frontmatter + Footnotes
```

### 各ステージの詳細

#### 1. HTML Sanitizer

- 全てのHTMLタグをエスケープ (`<tag>` → `&lt;tag&gt;`)
- HTMLエンティティ（`&nbsp;`, `&lt;`等）は保持
- XSS攻撃の防止

#### 2. Conflict Resolver (前処理)

- UMD構文を`{{MARKER:...:MARKER}}`形式で一時保護
- Markdown構文との衝突を回避
- カスタムヘッダーID `{#id}` を抽出・除去

#### 3. Frontmatter Extractor

- YAML (`---`) またはTOML (`+++`) フロントマターを検出
- 本文から分離し、メタデータとして保存
- HTML出力には含めない

#### 4. comrak Parser

- CommonMark準拠のMarkdownパース
- AST（Abstract Syntax Tree）を構築
- GFM拡張機能（テーブル、打ち消し線等）をサポート

#### 5. UMD Extensions

- UMD独自構文（強調、装飾、プラグイン等）をASTに追加
- セル連結対応テーブルをパース
- Bootstrapクラスへのマッピング

#### 6. HTML Renderer

- ASTを型安全なHTML文字列に変換
- セマンティックHTMLタグを優先的に使用
- Bootstrapユーティリティクラスを自動生成

#### 7. Plugin Processor

- プラグイン構文を`<template>`タグに変換
- 引数を`<data value="index">`要素に格納
- バックエンドでの処理に最適化

#### 8. Post Processor (後処理)

- 保護されたマーカーを実際のHTMLに復元
- カスタムヘッダーIDを`<h*>`タグに適用
- 最終的なHTML出力を生成

---

## 主要コンポーネント

### src/lib.rs

- メインエントリポイント
- `parse()` 関数: テキスト → HTML変換
- `ParseResult` 構造体: HTML本文、フロントマター、脚注を返す

### src/parser.rs

- comrakベースのMarkdownパーサー
- CommonMark + GFM拡張機能をサポート
- ASTの構築と基本的な変換処理

### src/sanitizer.rs

- HTML安全化モジュール
- ユーザー入力のHTMLエスケープ
- エンティティの保持ロジック
- XSS脆弱性の防止

### src/frontmatter.rs

- フロントマター抽出モジュール
- YAML/TOML形式をサポート
- メタデータとして本文と分離

### src/extensions/

- UMD拡張機能モジュール群

#### src/extensions/conflict_resolver.rs

- 構文衝突解決
- マーカーベース前処理・後処理
- カスタムヘッダーID処理

#### src/extensions/emphasis.rs

- UMD強調構文: `''bold''`, `'''italic'''`
- 視覚的タグ（`<b>`, `<i>`）を生成

#### src/extensions/block_decorations.rs

- ブロック装飾プレフィックス: `COLOR()`, `SIZE()`, `CENTER:` 等
- Bootstrapクラスへのマッピング

#### src/extensions/inline_decorations.rs

- インライン装飾関数: `&color()`, `&badge()`, `&ruby()` 等
- セマンティックHTML要素の生成
- 取り消し線: `%%text%%` → `<s>text</s>`

#### src/extensions/plugins.rs

- プラグインシステム実装
- インライン型: `&function(...)`
- ブロック型: `@function(...)`
- `<template>`タグによるSSR最適化

#### src/extensions/table/

- テーブル機能統合モジュール

##### src/extensions/table/umd/parser.rs

- UMDテーブルパーサー
- セル連結検出

##### src/extensions/table/umd/cell_spanning.rs

- colspan/rowspan処理
- `|>` (横連結), `|^` (縦連結)

##### src/extensions/table/umd/decorations.rs

- テーブルセル装飾
- 配置、色、サイズ等のスタイリング

---

## 依存クレート

### プロダクション依存

```toml
[dependencies]
wasm-bindgen = "0.2.108"        # WASM bindings
comrak = "0.50.0"               # Markdown parser (GFM)
ammonia = "4.1.2"               # HTML sanitization
maud = "0.27.0"                 # Type-safe HTML generation
regex = "1.12.2"                # Pattern matching
once_cell = "1.21.3"            # Lazy static initialization
unicode-segmentation = "1.12.0" # Grapheme cluster handling
html-escape = "0.2.13"          # HTML escaping
base64 = "0.22.1"               # Base64 encoding
serde_json = "1.0.149"          # JSON serialization
```

### 開発依存

```toml
[dev-dependencies]
insta = "1.46.2"             # Snapshot testing
criterion = "0.8.1"          # Benchmarking
wasm-bindgen-test = "0.3.58" # WASM testing
```

### 各クレートの役割

- **comrak**: CommonMark + GFM準拠のMarkdownパーサー、AST生成
- **ammonia**: HTML安全化、ホワイトリストベースのサニタイゼーション
- **maud**: 型安全なHTML生成、コンパイル時検証
- **regex**: 正規表現マッチング、UMD構文検出
- **once_cell**: 遅延初期化、正規表現パターンのキャッシュ
- **base64**: プラグイン引数のエンコーディング（現在は不使用、将来的に削除予定）
- **wasm-bindgen**: WebAssembly対応、ブラウザでの実行

---

## ディレクトリ構造

```
umd/
├── Cargo.toml              # プロジェクト設定
├── build.sh                # WASMビルドスクリプト
├── README.md               # プロジェクト概要
├── PLAN.md                 # 実装計画（未実装機能）
├── WASM_BUILD.md           # WASMビルドガイド
├── docs/                   # ドキュメント
│   ├── implemented-features.md  # 実装済み機能
│   ├── planned-features.md      # 実装予定機能
│   └── architecture.md          # このドキュメント
├── src/                    # ソースコード
│   ├── lib.rs              # メインエントリポイント
│   ├── parser.rs           # Markdownパーサー
│   ├── sanitizer.rs        # HTML安全化
│   ├── frontmatter.rs      # フロントマター処理
│   └── extensions/         # UMD拡張機能
│       ├── mod.rs
│       ├── emphasis.rs
│       ├── block_decorations.rs
│       ├── inline_decorations.rs
│       ├── plugins.rs
│       ├── conflict_resolver.rs
│       └── table/
│           ├── mod.rs
│           └── umd/
│               ├── mod.rs
│               ├── parser.rs
│               ├── cell_spanning.rs
│               └── decorations.rs
├── tests/                  # 統合テスト
│   ├── commonmark.rs       # CommonMark準拠テスト
│   ├── bootstrap_integration.rs  # Bootstrap統合テスト
│   ├── conflict_resolution.rs    # 構文衝突テスト
│   └── test_semantic_integration.rs  # セマンティックHTML
├── examples/               # サンプル・デモ
│   ├── test_output.rs
│   ├── test_bootstrap_integration.rs
│   ├── test_frontmatter.rs
│   ├── test_footnotes.rs
│   ├── test_header_id.rs
│   ├── test_plugin_extended.rs
│   ├── test_simple_umd.rs
│   ├── test_umd_header.rs
│   ├── test_table_colspan.rs
│   └── ... (その他のデモ)
└── target/                 # ビルド成果物
```

---

## 構文優先順位ポリシー

### 競合時の解決ルール

複数の構文が競合する場合、以下の優先順位で解決します。

#### 1. ブロック引用

- **UMD形式優先**: `> ... <` （閉じタグ検出時）
- **Markdown形式**: 閉じタグなし → `>` 行頭プレフィックス

#### 2. 強調表現

- **両スタイルサポート（共存）**:
  - Markdown: `*em*`, `**strong**` → セマンティックタグ (`<em>`, `<strong>`)
  - UMD: `'''italic'''`, `''bold''` → 視覚的タグ (`<i>`, `<b>`)
- **違い**: アクセシビリティやSEOへの影響が異なる

#### 3. 取り消し線

- **両スタイルサポート（共存）**:
  - Markdown/GFM: `~~text~~` → `<del>text</del>` (削除された内容)
  - UMD: `%%text%%` → `<s>text</s>` (正確でなくなった内容)
- **構文が明確に異なるため矛盾なし**

#### 4. リストマーカー

- **両スタイルサポート**:
  - `-`, `*` → 順序なしリスト
  - `+`, `1.` → 順序付きリスト

#### 5. 水平線

- `----` (4+文字) 優先
- `***`, `___` も対応
- **CommonMark準拠**

#### 6. テーブル

- **UMD形式とMarkdown形式を構文で判別**:
  - UMD: `|>`または`|^`を含む（セル連結対応）
  - Markdown: 区切り行 `|---|` を含む（ソート可能）
- **構文が明確に異なるため矛盾なし**

#### 7. プラグイン構文と@mention

- **プラグイン**: `@function()` - 括弧必須
- **@mention**: `@username` - 括弧なし
- **括弧の有無で明確に区別可能**

### Markdown仕様との矛盾箇所まとめ

| UMD構文       | Markdown構文        | 矛盾度 | 解決策                   |
| ------------- | ------------------- | ------ | ------------------------ |
| `'''text'''`  | `***text***`        | 中     | 3連続クォートを優先検出  |
| `+ item`      | `+ item` (一部方言) | 低     | 順序付きリストとして統一 |
| `COLOR(...):` | `: definition`      | 低     | 大文字キーワードで判別   |
| `> ... <`     | `> quote`           | 低     | 閉じタグで判別           |
| `%%text%%`    | `~~text~~`          | 低     | 異なる構文で明確に区別   |
| `@function()` | `@mention`          | 低     | 括弧の有無で区別         |

**対策**:

- パーサーの優先順位で明示的に処理
- conflict_resolverで包括的にテスト
- 曖昧な入力に対する警告メッセージ（将来実装予定）

---

## 開発者向けガイド

このセクションはAIエージェントおよび開発者向けの実装ハンドブックです。

### ドキュメント駆動型開発

**原則**: 実装前に仕様を文書化し、テストと仕様の一貫性を保つ

#### ワークフロー

1. `PLAN.md` を読んで実装対象のタスクを理解
2. 新しいルール（タグ使法等）が生じた場合は、`docs/rules/` に新規文書を作成するか、既存文書を更新
3. **実装完了の定義**: コードが書かれ、テストがパスし、仕様が `docs/` に更新されたとき

#### ドキュメント管理

- `PLAN.md` が100行を超える場合は、実装済みセクションを `docs/archive/` に移動するか削除
- 複数のAIエージェント（Gemini, Grok等）での理解を想定した明確な文書を作成

### 何をどこで変更するか

#### 構文競合やUMD仕様の修正

ファイル: `src/extensions/conflict_resolver.rs` + `tests/conflict_resolution.rs`

- 機能: UMD構文とMarkdown構文の衝突解決
- マーカー方式での前処理・後処理
- カスタムヘッダーID処理（`{#id}`）

#### インライン装飾・ブロック装飾

- **インライン**: `src/extensions/inline_decorations.rs`
  - `&color()`, `&badge()`, `&ruby()` などのセマンティック関数
  - 取り消し線 `%%text%%` → `<s>`
- **ブロック**: `src/extensions/block_decorations.rs`
  - `COLOR(...)`, `SIZE(...)`, `CENTER:` などのプレフィックス装飾
  - LEFT/CENTER/RIGHT/JUSTIFYプレフィックス（配置制御）

#### プラグインシステム

- **メイン実装**: `src/extensions/plugins.rs`
- **マーカー補助**: `src/extensions/plugin_markers.rs`
- **構文**: `&fn(args){...};` (インライン), `@fn(args){{ ... }}` (ブロック)
- **出力形式**: `<template class="umd-plugin umd-plugin-*"><data value="i"></data>...</template>`
- **実行**: 外部（Nuxt/Laravel等のバックエンド）で処理

#### コードブロック機能

ファイル: `src/extensions/code_block.rs`

- 言語別シンタックスハイライト: `language-*` クラス
- Mermaid図: `<figure class="code-block code-block-mermaid mermaid-diagram">...</figure>` でラップ（内部にSVGを直接配置）
- プレーンテキスト: 言語指定なし → `<pre>...</pre>`
- 仕様: `pre`タグには`lang`属性を付与しない（言語情報は`code.language-*`へ統一）

#### テーブル拡張

ファイル: `src/extensions/table/umd/*`

- `|>` (セル横連結), `|^` (セル縦連結)
- セル装飾: 配置、色、サイズ
- ComraKのテーブルAS Tを UMD仕様で拡張

#### メディア自動検出

ファイル: `src/extensions/media.rs`

- 画像構文から動画・音声・ダウンロードリンク自動判別
- `![alt](video.mp4)` → `<video>`
- `![alt](audio.mp3)` → `<audio>`
- `![alt](image.png)` → `<picture>` (応答性対応)
- ブロック vs インライン自動判別

### プロジェクト固有の規約

#### パイプラインの安定性

- **ルール**: パイプラインの優先順位を維持する（ローカル「整理」より重視）
- **理由**: 複数の機能が前処理・後処理ステージに依存している
- **チェック方法**: `src/lib.rs:parse_with_frontmatter_opts()` の処理順序表を参照

#### コード保護パターン

- **コード区間の保護**: 正規表現変換前に `protect_code_sections` で保護
- **新規regex**: 既存の保護パターンを回避しない設計

#### UMM固有構文

- **ブロック引用**: `> ... <` → `<blockquote class="umd-blockquote">`（Plain Markdownと異なる）
- **下線**: `__text__` → `<u>` （Discord風、CommonMark strong emphasisではない）
- **プラグイン出力**: メタデータ優先HTML（`<template>`タグ）、実行はバックエンド
- **Base URIリライト**: `ParserOptions.base_url` での opt-in（`tests/base_url.rs`に仕様あり）

### ビルド・テスト・デバッグワークフロー

#### ローカルCI実行

```bash
cargo build --verbose && cargo test --verbose
```

`.github/workflows/rust.yml` に合わせた本番環境相当の実行。

#### 高速検証（開発中）

```bash
# 特定テストファイルのみ実行
cargo test --test conflict_resolution
cargo test --test bootstrap_integration

# 特定モジュールのテストを実行
cargo test transform_images_to_media -- --nocapture
```

#### WASMビルド

```bash
./build.sh [dev|release]
# 出力: pkg/ フォルダ
# 要件: wasm-pack ツールチェーン
```

#### 手動デバッグ例実行

```bash
cargo run --example test_plugin_extended
cargo run --example test_table_colspan
```

`examples/` フォルダのサンプルで機能を素早く検証。

### パイプライン処理順序（重要）

`src/lib.rs:parse_with_frontmatter_opts()` の実行順序は厳密であり、多くの機能がこの順番に依存しています。

1. **フロントマター抽出** - メタデータを先に取得
2. **前処理器** - ネストブロック、タスクリスト、下線などの一次処理
3. **競合保護** - UMD構文をマーカーでラップ → Markdown競合を回避
4. **サニタイズ** - HTML直接入力をエスケープ
5. **comrakパース** - Markdown → AST生成
6. **拡張機能** - インライン/ブロック装飾、プラグイン、メディア処理
7. **フットノート抽出** - 脚注をJSON化
8. **後処理** - マーカーの復元、最終HTML出力

**順序の変更は避けてください。テストで順序依存がないことが確認される場合にのみ、慎重に変更してください。**

### 依存クレートの役割（実装時の参考）

- **comrak**: CommonMark + GFM パーサー、AST生成で一度だけ使用
- **regex**: UMD構文検出・変換、複数ステージで使用
- **ammonia**: ホワイトリストベースのHTML安全化
- **maud**: 型安全HTML生成（コンパイル時検証）
- **once_cell**: 正規表現パターンのキャッシュ（性能最適化）

## セキュリティ方針

### HTML入力制限

**原則**: 直接HTML入力は**完全禁止**

#### 実装

1. 入力時に全てのHTMLタグをエスケープ
2. HTMLエンティティ（`&nbsp;`, `&lt;`等）のみ保持
3. パーサー生成HTMLのみ出力に使用
4. XSS攻撃ベクトルの完全遮断

#### 例外

プラグイン出力のHTMLは許可:

- プラグインが生成するHTMLは信頼されたコードとして扱う
- プラグイン側でサニタイズ責任を負う
- ユーザー入力をプラグインに渡す場合は、プラグイン内でエスケープ必須

### URL Sanitization

#### 禁止するスキーム（ブラックリスト方式）

XSS対策のため、以下のスキームをブロック:

- `javascript:` - JavaScript実行による直接的なXSS攻撃
- `data:` - Base64エンコードされたスクリプト埋め込み
  - 例: `data:text/html,<script>alert('XSS')</script>`
- `vbscript:` - VBScript実行（IEレガシー対策）
- `file:` - ローカルファイルシステムアクセス（情報漏洩リスク）

#### 許可するスキーム

上記以外の全てのスキーム:

- HTTP/HTTPS: `http:`, `https:`
- メール/通信: `mailto:`, `tel:`, `sms:`
- FTP: `ftp:`, `ftps:`
- カスタムアプリスキーム: `spotify:`, `steam:`, `discord:`, `slack:`, `zoom:`, `vscode:` 等
- その他: 相対パス、ルート相対パス、アンカー（`#`）

#### 検出方法

正規表現で`^(javascript|data|vbscript|file):`を検出（大文字小文字区別なし）

#### 処理

禁止スキームが検出された場合、URLを空文字列または安全なプレースホルダー（`#blocked-url`）に置換

---

## パフォーマンス考慮事項

### 最適化戦略

1. **遅延初期化**: 正規表現パターンを`once_cell`でキャッシュ
2. **AST再利用**: comrakのASTを効率的に変換
3. **メモリ効率**: 不要なクローンを最小化
4. **並列処理**: 将来的にRayonによる並列化を検討

### ベンチマーク目標

- **小規模文書** (1KB): < 1ms
- **中規模文書** (10KB): < 10ms
- **大規模文書** (100KB): < 100ms

---

## テスト戦略

### テストカテゴリ

1. **Unit Tests** (121 tests): 各モジュールの個別機能テスト
2. **Bootstrap Integration Tests** (22 tests): Bootstrapクラス生成の検証
3. **CommonMark Compliance Tests** (18 tests): CommonMark仕様への準拠
4. **Conflict Resolution Tests** (13 tests): 構文衝突の解決
5. **Semantic Integration Tests** (1 test): セマンティックHTML生成
6. **Doctests** (9 tests): ドキュメント内のサンプルコード

### テストツール

- **insta**: スナップショットテスト
- **criterion**: パフォーマンスベンチマーク
- **wasm-bindgen-test**: WASMテスト

---

## 将来の拡張計画

### 短期（1-2ヶ月）

- メディアファイル自動検出の実装
- JUSTIFY, TRUNCATEプレフィックスの実装
- スポイラー機能の実装
- 定義リストの実装

### 中期（3-6ヶ月）

- テーブルバリエーション（striped, hover等）
- リスト内ブロック要素のサポート
- タスクリスト拡張
- カスタムリンク属性

### 長期（6ヶ月以降）

- パフォーマンス最適化
- プラグインAPI公開
- 多言語対応強化
- ビジュアルエディタ統合

---

## 参考リソース

- **PHP実装**: https://github.com/logue/LukiWiki/tree/master/app/LukiWiki
- **仕様書**: https://github.com/logue/LukiWiki-core/blob/master/docs/rules.md
- **CommonMark仕様**: https://spec.commonmark.org/
- **GFM仕様**: https://github.github.com/gfm/
- **Bootstrap 5**: https://getbootstrap.com/docs/5.3/

---

## ライセンス

MIT License
