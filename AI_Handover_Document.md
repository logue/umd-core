# Universal Markdown (UMD) Core - AI引き継ぎドキュメント

> **注意**: この文書は引き継ぎ時点のスナップショットです。最新仕様・処理順序は [docs/README.md](docs/README.md) と [docs/architecture.md](docs/architecture.md) を参照してください。

**作成日**: 2026年2月16日  
**最終更新**: 2026年2月18日  
**対象プロジェクト**: umd-core (Rustライブラリ)  
**目的**: VSCode拡張機能およびWASMプレビュー表示ツールの開発のための技術引き継ぎ

---

## プロジェクト概要

Universal Markdown (UMD) は、CommonMark準拠のMarkdownパーサーをベースに、Bootstrap 5統合、セマンティックHTML要素、拡張可能なプラグインシステムを備えたポストMarkdownパーサーです。Rustで実装されており、WASM出力にも対応しています。

### 主な特徴

- **CommonMark準拠**: 標準Markdown構文の高い互換性（75%+テストパス率）
- **Bootstrap 5統合**: デフォルトでBootstrapクラスを生成（Core UI互換）
- **セマンティックHTML**: アクセシビリティとSEOに優しいHTML生成
- **拡張構文**: メディア自動検出、Spoiler、定義リスト、テーブル拡張、ブロック/インライン装飾
- **プラグインシステム**: インライン型（`&function(args){content};`）とブロック型（`@function(args){{ content }}`）
- **セキュリティ**: HTMLサニタイゼーション、危険URLスキームブロック
- **WASM対応**: WebAssembly出力によるブラウザ実行
- **UMDレガシー互換**: 既存PHP実装との後方互換性

### 技術スタック

- **言語**: Rust 1.93.1 (Edition 2024)
- **ベースパーサー**: comrak 0.50.0 (GFM準拠ASTベース)
- **HTML生成**: maud 0.27.0 (型安全)
- **HTML安全化**: ammonia 4.1.2
- **正規表現**: regex 1.12.3
- **WASM**: wasm-bindgen 0.2.108

---

## アーキテクチャ概要

### 処理フロー

```text
Input Text
    ↓
[HTML Sanitizer] - HTMLエスケープ、エンティティ保持
    ↓
[Conflict Resolver] - UMD構文をマーカーで保護
    ↓
[Frontmatter Extractor] - YAML/TOMLフロントマター抽出
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

### 主要コンポーネント

- **lib.rs**: メインAPI（`parse()`, `parse_with_frontmatter()`）
- **parser.rs**: メインのパース処理
- **sanitizer.rs**: HTML安全化
- **frontmatter.rs**: フロントマター処理
- **extensions/**: UMD拡張機能
  - **block_decorations.rs**: ブロック装飾（色、サイズ、配置）
  - **inline_decorations.rs**: インライン装飾（バッジ、ルビ、上付き/下付き）
  - **media.rs**: メディアファイル自動検出
  - **plugins.rs**: プラグインシステム
  - **table/**: 拡張テーブル機能
  - **conflict_resolver.rs**: 構文競合解決
  - **preprocessor.rs**: 前処理

---

## 主要API

### 基本パース関数

```rust
use umd::parse;

let markdown = "# Hello World\n\nThis is **bold** text.";
let html = parse(markdown);
```

### フロントマター付きパース

```rust
use umd::parse_with_frontmatter;

let input = "---\ntitle: Test\n---\n\n# Content";
let result = parse_with_frontmatter(input);

println!("HTML: {}", result.html);
if let Some(fm) = result.frontmatter {
    println!("Title: {}", fm.content["title"]);
}
```

### フットノート取得

```rust
let result = parse_with_frontmatter(input);
println!("Footnotes: {:?}", result.footnotes); // JSON配列
```

---

## UMD拡張構文の概要

### ブロック装飾

```markdown
{COLOR:red}赤いブロック{/COLOR}
{SIZE:large}大きなテキスト{/SIZE}
{ALIGN:center}中央揃え{/ALIGN}
```

### インライン装飾

```markdown
バッジ: &badge(primary){New};
ルビ: &ruby(漢字){かんじ};
上付き: &sup(2);
下付き: &sub(2);
```

### プラグインシステム

**インライン型**:

```markdown
&function(args){content};
```

**ブロック型**:

```markdown
@function(args){{
content
}}
```

### メディア自動検出

```markdown
![動画](video.mp4) → <video>タグ
![音声](audio.mp3) → <audio>タグ
![画像](image.jpg) → <picture>タグ
```

### Spoiler機能

```markdown
||ネタバレ内容||
```

### 定義リスト

```markdown
:用語|定義文
:用語2|
複数行定義

- リストも可
```

### 拡張テーブル

```markdown
|LEFT 左揃え|CENTER 中央|RIGHT 右揃え|
|=== ヘッダー ===|===|===|
|通常セル|通常|通常|
```

---

## ビルド方法

### 通常ビルド

```bash
cargo build
```

### リリースビルド

```bash
cargo build --release
```

### WASMビルド

```bash
# wasm-packインストール
cargo install wasm-pack

# リリースビルド
./build.sh release
# または
./build.sh

# 開発ビルド
./build.sh dev
```

WASMビルド後、`pkg/`ディレクトリに以下のファイルが生成：

- `umd.js` - JavaScriptバインディング
- `umd.d.ts` - TypeScript型定義
- `umd_bg.wasm` - WASMバイナリ
- `package.json` - npmパッケージ情報

### テスト実行

```bash
cargo test
```

---

## VSCode拡張機能開発のためのポイント

### 拡張機能の役割

- UMD構文のハイライト表示
- リアルタイムプレビュー
- 構文チェック
- スニペット提供

### 実装アプローチ

1. **Language Server Protocol (LSP)**: 構文解析とエラー検出
2. **TextMate Grammar**: 構文ハイライト
3. **Webview API**: プレビュー表示（WASM統合）

### 必要なファイル構造

```text
vscode-umd-extension/
├── package.json
├── syntaxes/
│   └── umd.tmLanguage.json  # TextMate文法
├── server/
│   └── src/
│       └── server.ts        # LSPサーバー
├── client/
│   └── src/
│       └── extension.ts     # 拡張機能メイン
└── webview/
    └── preview.html         # プレビューUI
```

### WASM統合

VSCode拡張機能内でWASMを使用する場合：

- `wasm-pack`で生成したファイルを拡張機能にバンドル
- Webview内でWASMをロードしてパース実行
- セキュリティのため、Webviewは制限された環境で実行

---

## WASMプレビュー表示ツール開発のためのポイント

### ツールの役割

- ブラウザベースのUMDプレビュー
- リアルタイム編集・プレビュー
- スタンドアロンWebアプリケーション

### 実装アプローチ（WASMプレビュー）

1. **HTML/CSS/JS**: シンプルなWebページ
2. **WASM統合**: パース処理をWASMで実行
3. **リアルタイム更新**: textareaの入力で即時プレビュー

### 基本構造

```html
<!DOCTYPE html>
<html>
  <head>
    <title>UMD Preview</title>
    <link
      href="https://cdn.jsdelivr.net/npm/bootstrap@5.3.0/dist/css/bootstrap.min.css"
      rel="stylesheet"
    />
  </head>
  <body>
    <div class="container-fluid">
      <div class="row">
        <div class="col-md-6">
          <textarea id="input" class="form-control" rows="20"></textarea>
        </div>
        <div class="col-md-6">
          <div id="preview" class="border p-3"></div>
        </div>
      </div>
    </div>

    <script type="module">
      import init, { parse } from "./pkg/umd.js";

      await init();

      const input = document.getElementById("input");
      const preview = document.getElementById("preview");

      input.addEventListener("input", () => {
        const html = parse(input.value);
        preview.innerHTML = html;
      });
    </script>
  </body>
</html>
```

### 高度な機能

- **フロントマター表示**: YAML/TOMLメタデータを別パネルで表示
- **フットノート処理**: 構造化データをHTMLに変換
- **プラグイン実行**: JavaScriptでプラグインを処理
- **テーマ切り替え**: Bootstrapテーマ変更
- **エクスポート**: HTML/PDF出力

---

## 拡張・カスタマイズのポイント

### プラグインシステムの活用

UMDのプラグインシステムは拡張可能：

- **標準プラグイン**: badge, ruby, sup, sub, kbd, cite, abbr など
- **カスタムプラグイン**: 独自関数を追加可能

プラグイン実装例：

```rust
// plugins.rs に追加
fn custom_plugin(args: &str, content: &str) -> String {
    format!("<div class=\"custom\">{}</div>", content)
}
```

### 設定オプション

`ParserOptions` 構造体で動作をカスタマイズ：

- `allow_file_scheme`: file:// URL許可
- `bootstrap_version`: Bootstrapバージョン指定
- `semantic_html`: セマンティックHTML有効化

### セキュリティ考慮

- HTMLサニタイズは常に有効
- 危険URLスキームはブロック
- WASM実行はサンドボックス環境

---

## 関連ドキュメント

- **[README.md](README.md)**: プロジェクト概要と使用例
- **[docs/implemented-features.md](docs/implemented-features.md)**: 実装済み機能詳細
- **[docs/architecture.md](docs/architecture.md)**: 技術アーキテクチャ
- **[docs/planned-features.md](docs/planned-features.md)**: 計画機能
- **[PLAN.md](PLAN.md)**: 開発計画と仕様変更履歴
- **[WASM_BUILD.md](WASM_BUILD.md)**: WASMビルド手順

---

## 注意事項

- **CommonMark準拠**: 標準Markdownとの互換性を維持
- **セキュリティ優先**: HTML直接入力は禁止
- **拡張性**: プラグインシステムで機能を追加
- **パフォーマンス**: WASMビルド時はサイズ最適化を考慮
- **テスト**: 変更時は必ずテスト実行

このドキュメントを基に、VSCode拡張機能とWASMプレビュー表示ツールの開発を進めてください。必要に応じて、元のリポジトリのコードを参照しつつ、別リポジトリでの実装を行ってください。
