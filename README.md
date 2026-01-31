# Universal Markdown

CommonMark準拠のMarkdownパーサーに、Bootstrap 5統合、セマンティックHTML要素、拡張可能なプラグインシステムを備えたポストMarkdownパーサーです。LukiWikiレガシー構文との互換性もサポートします。

## 特徴

- **CommonMark準拠**: 標準Markdown構文の高い互換性
- **ポストMarkdown**: Markdownを超える拡張機能
- **Bootstrap 5統合**: デフォルトでBootstrapクラスを生成（Core UI互換）
- **セマンティックHTML**: アクセシビリティとSEOに優しいHTML生成
- **Definition Lists**: 用語辞典構文のサポート
- **LukiWiki互換**: レガシーPHP実装との後方互換性
- **フロントマターサポート**: YAML/TOML形式のメタデータ
- **フットノート**: 標準的な脚注構文のサポート
- **セキュリティ**: HTMLサニタイゼーションによるXSS対策
- **WASM対応**: WebAssembly出力によるブラウザ実行
- **拡張性**: プラグインシステムによる機能拡張

## フロントマター

ドキュメントの先頭にYAMLまたはTOML形式のメタデータを配置できます。

### YAML形式

```markdown
---
title: ドキュメントタイトル
author: 著者名
date: 2024-01-23
tags:
  - rust
  - wiki
---

# コンテンツ
```

### TOML形式

```markdown
+++
title = "ドキュメントタイトル"
author = "著者名"
date = 2024-01-23
+++

# コンテンツ
```

フロントマターはHTML出力から除外され、`parse_with_frontmatter()`関数を使用することで別途取得できます。

```rust
use universal_markdown::parse_with_frontmatter;

let input = "---\ntitle: Test\n---\n\n# Content";
let result = parse_with_frontmatter(input);

if let Some(fm) = result.frontmatter {
    println!("Format: {:?}", fm.format); // Yaml
    println!("Content: {}", fm.content);
}
println!("HTML: {}", result.html);
```

## フットノート（脚注）

Markdownの標準的なフットノート構文をサポートしています：

```markdown
本文にフットノート[^1]を含めます。

別の段落で別のフットノート[^note2]を参照。

[^1]: これが最初のフットノートです。

[^note2]: 名前付きフットノートも使えます。
```

フットノートは本文から分離され、`ParseResult`の`footnotes`フィールドで取得できます：

```rust
use universal_markdown::parse_with_frontmatter;

let input = "Text with footnote[^1].\n\n[^1]: Footnote content.";
let result = parse_with_frontmatter(input);

println!("Body: {}", result.html);
if let Some(footnotes) = result.footnotes {
    println!("Footnotes: {}", footnotes);
}
```

フットノートは`<section class="footnotes">`として生成され、適切にスタイリングできます。

## Bootstrap 5統合

LukiWiki-rsは、デフォルトでBootstrap 5のクラスを生成します。これにより、CoreUIなどのBootstrapベースのフレームワークとシームレスに統合できます。

### デフォルトクラス

特定のHTML要素には、自動的にBootstrapクラスが適用されます：

- **テーブル**: `<table class="table">`
- **ブロック引用**: `<blockquote class="blockquote">` (Markdown標準) / `<blockquote class="lukiwiki">` (LukiWiki形式)

### ブロック装飾プレフィックス

行の先頭にプレフィックスを付けることで、Bootstrapクラスやスタイルを適用できます：

```markdown
COLOR(primary): プライマリカラーのテキスト
SIZE(2): 大きいテキスト (fs-2)
CENTER: 中央寄せのテキスト
SIZE(1.5): COLOR(danger): RIGHT: 複合スタイル
```

#### サポートされるプレフィックス

- **COLOR(value)**: Bootstrapカラークラス (`text-{color}`) または任意の色値
  - Bootstrap色: `primary`, `secondary`, `success`, `danger`, `warning`, `info`, `light`, `dark`
  - 例: `COLOR(primary): テキスト` → `<p class="text-primary">テキスト</p>`
  - カスタム色: `COLOR(#FF0000): 赤` → `<p style="color: #FF0000">赤</p>`

- **SIZE(value)**: Bootstrapフォントサイズクラス (`fs-{1-6}`) または任意のサイズ
  - Bootstrap: `2.5` → `fs-1`, `2` → `fs-2`, `1.75` → `fs-3`, `1.5` → `fs-4`, `1.25` → `fs-5`, `0.875` → `fs-6`
  - 例: `SIZE(1.5): テキスト` → `<p class="fs-4">テキスト</p>`
  - カスタム: `SIZE(3rem): 大きい` → `<p style="font-size: 3rem">大きい</p>`

- **配置**: `LEFT:`, `CENTER:`, `RIGHT:` → `text-start`, `text-center`, `text-end`

- **複合**: 複数のプレフィックスを組み合わせ可能
  - 例: `SIZE(2): COLOR(primary): CENTER: テキスト`

### インライン装飾関数

インラインでBootstrapクラスを適用できます：

```markdown
&color(primary){重要なテキスト};
&size(1.5){やや大きいテキスト};
&badge(danger){Error};
&badge(success-pill){Active};
```

#### サポートされる関数

- **&color(fg,bg){text};**: テキスト色・背景色
  - 例: `&color(primary){テキスト};` → `<span class="text-primary">テキスト</span>`
  - 例: `&color(primary,primary-subtle){テキスト};` → `<span class="text-primary bg-primary-subtle">テキスト</span>`

- **&size(value){text};**: フォントサイズ
  - 例: `&size(1.5){テキスト};` → `<span class="fs-4">テキスト</span>`

- **&badge(type){text};**: Bootstrapバッジ
  - 基本: `&badge(primary){New};` → `<span class="badge bg-primary">New</span>`
  - Pill: `&badge(success-pill){Active};` → `<span class="badge rounded-pill bg-success">Active</span>`
  - リンク: `&badge(danger){[Error](/error)};` → `<a href="/error" class="badge bg-danger">Error</a>`

## テーブル

Universal Markdownは、GFM（GitHub Flavored Markdown）標準のテーブルに加えて、LukiWiki拡張機能をサポートしています。

### 基本テーブル（GFM準拠）

```markdown
| Header1 | Header2 | Header3 |
| ------- | ------- | ------- |
| Cell1   | Cell2   | Cell3   |
| Cell4   | Cell5   | Cell6   |
```

出力：

```html
<table class="table">
  <thead>
    <tr>
      <th>Header1</th>
      <th>Header2</th>
      <th>Header3</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td>Cell1</td>
      <td>Cell2</td>
      <td>Cell3</td>
    </tr>
    <tr>
      <td>Cell4</td>
      <td>Cell5</td>
      <td>Cell6</td>
    </tr>
  </tbody>
</table>
```

**特徴**：

- 自動的にBootstrapの`table`クラスが付与されます
- GFMテーブルとLukiWikiテーブルの両方で`<thead>`と`<tbody>`が正しく生成されます

### LukiWiki拡張：セル連結

Markdownの標準テーブルでは表現できないセル連結をサポートします。

#### 横方向連結（colspan）

`|>`マーカーを使用して、右のセルと連結します：

```markdown
| Header1 |> | Header3 |
| Cell1 | Cell2 | Cell3 |
```

出力：

```html
<table class="table" data-lukiwiki="true">
  <thead>
    <tr>
      <th colspan="2">Header1</th>
      <th>Header3</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td>Cell1</td>
      <td>Cell2</td>
      <td>Cell3</td>
    </tr>
  </tbody>
</table>
```

#### 縦方向連結（rowspan）

`|^`マーカーを使用して、上のセルと連結します：

```markdown
| Header1 | Header2 |
| Cell1 | Cell2 |
| |^ | Cell3 |
```

出力：

```html
<table class="table" data-lukiwiki="true">
  <thead>
    <tr>
      <th>Header1</th>
      <th>Header2</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td rowspan="2">Cell1</td>
      <td>Cell2</td>
    </tr>
    <tr>
      <td>Cell3</td>
    </tr>
  </tbody>
</table>
```

#### 複合連結

colspanとrowspanを組み合わせることもできます：

```markdown
| Header1 |> | Header3 |
| Cell1 |> | Cell3 |
| |^ |^ | Cell4 |
```

### セル内装飾

#### 色指定

`COLOR(fg,bg):`プレフィックスでセルの前景色・背景色を指定できます：

```markdown
| COLOR(primary): Header | COLOR(,success): Cell |
| Normal | COLOR(danger,warning): Alert |
```

- Bootstrap色名（`primary`, `secondary`, `success`, `danger`, `warning`, `info`, `light`, `dark`）は自動的に`text-*`/`bg-*`クラスに変換
- カスタムカラーコードも使用可能（インラインスタイルとして出力）
- 前景色のみ、背景色のみの指定も可能（`,`で区切る）

#### サイズ指定

`SIZE(value):`プレフィックスでフォントサイズを指定できます：

```markdown
| SIZE(1.5): Large | SIZE(0.8): Small |
```

#### 配置指定

##### 水平配置

```markdown
| LEFT: Left | CENTER: Center | RIGHT: Right |
```

- `LEFT:` → `text-start`（左寄せ）
- `CENTER:` → `text-center`（中央寄せ）
- `RIGHT:` → `text-end`（右寄せ）
- `JUSTIFY:` → `text-justify`（両端揃え）

##### 垂直配置

##### 垂直配置

テーブルセル内でBootstrap配置クラスを使用できます：

```markdown
| TOP: ヘッダー | MIDDLE: データ |
| ------------- | -------------- |
| BOTTOM: A     | BASELINE: B    |
```

- `TOP:` → `align-top`（上揃え）
- `MIDDLE:` → `align-middle`（中央揃え）
- `BOTTOM:` → `align-bottom`（下揃え）
- `BASELINE:` → `align-baseline`（ベースライン揃え）

#### 装飾の組み合わせ

複数の装飾を組み合わせることができます：

```markdown
| COLOR(primary): SIZE(1.2): CENTER: Header |
| RIGHT: Normal cell |
```

**重要な注意点**：

- GFM形式（2行目が`|---|---|`の形式）では、セル連結や装飾拡張は使用できません
- これらの拡張機能はLukiWiki形式のテーブル（2行目が区切り線でない）でのみ動作します
- LukiWikiテーブルには自動的に`data-lukiwiki="true"`属性が付与されます

### Definition Lists

用語と定義をセマンティックにマークアップできます：

```markdown
:HTML|HyperText Markup Language
:CSS|Cascading Style Sheets
```

出力:

```html
<dl>
  <dt>HTML</dt>
  <dd>HyperText Markup Language</dd>
  <dt>CSS</dt>
  <dd>Cascading Style Sheets</dd>
</dl>
```

## プラグインシステム

LukiWiki-rsは、拡張可能なプラグインシステムを提供します。プラグインは3つのパターンをサポートします。

### インライン型プラグイン

```
&function(args){content};
```

**出力HTML**:

```html
<span class="plugin-function" data-args="args">content</span>
```

**使用例**:

```
&highlight(yellow){重要なテキスト};
```

### ブロック型プラグイン（複数行）

```
@function(args){{ content }}
```

**出力HTML**:

```html
<div class="plugin-function" data-args="args">content</div>
```

**使用例**:

```
@toc(2){{
}}
```

### ブロック型プラグイン（単行）

```
@function(args){content}
```

**出力HTML**:

```html
<div class="plugin-function" data-args="args">content</div>
```

**使用例**:

```
@include(file.txt){デフォルトコンテンツ}
```

### プラグインのネストと再パース

プラグインは**ネスト可能**で、コンテンツ内にさらにプラグインを含めることができます：

```
&outer(arg1){text &inner(arg2){nested}; more};
```

**元のWiki構文がタグのテキストコンテンツとしてそのまま保持**されます。これにより、JavaScript側でプラグイン実行時に再パースが可能です：

```
@box(){{ **bold** and *italic* text }}
```

↓

```html
<div class="plugin-box" data-args="">**bold** and *italic* text</div>
```

プラグイン実装側でDOMのテキストコンテンツを取得し、再度LukiWikiパーサーに渡すことで、ネストされた構文も正しく処理できます。

**重要な特徴：**

- ブロック型プラグインは独立した`<div>`要素として出力され、`<p>`タグで括られません
- コンテンツ内の`&`文字は保持されるため、ネストされたプラグイン構文も再パース可能
- コンテンツ内のWiki構文（`**bold**`など）も生のまま保持されます

### 組み込み装飾との違い

LukiWiki-rsには、プラグインと同じ表記を使う**組み込み装飾関数**があります：

- `&color(fg,bg){text};` - 文字色・背景色
- `&size(rem){text};` - フォントサイズ
- `&sup(text);` - 上付き文字
- `&sub(text);` - 下付き文字
- `&lang(locale){text};` - 言語指定
- `&abbr(text){description};` - 略語説明

これらはパーサー内で直接HTMLに変換されます。組み込み装飾以外の名前は、すべて汎用プラグインとして処理されます。

## LukiWiki構文

## LukiWiki構文

### ヘッダーID

ヘッダーには自動的にURLセーフなIDが付与されます。すべてのIDには`h-`プレフィックスが付与され、システムIDとの競合を防ぎます。

**自動採番（デフォルト）**:

```markdown
# Introduction

## Details
```

生成されるHTML:

```html
<h1><a href="#h-1" id="h-1"></a>Introduction</h1>
<h2><a href="#h-2" id="h-2"></a>Details</h2>
```

**カスタムID（推奨）**:

```markdown
# Introduction {#intro}

## Details {#details}
```

生成されるHTML:

```html
<h1><a href="#h-intro" id="h-intro"></a>Introduction</h1>
<h2><a href="#h-details" id="h-details"></a>Details</h2>
```

**メリット**:

- ✅ URLセーフ（マルチバイト文字を避ける）
- ✅ 短いURL（SNSでの共有に最適）
- ✅ 安定したリンク（ヘッダーテキスト変更に強い）
- ✅ セキュリティ（同形異字による偽装を防止）
- ✅ ID競合の防止（`h-`プレフィックスでシステムIDと分離）

カスタムIDは`{#custom-id}`構文で指定します。指定がない場合は`h-1`, `h-2`のように自動採番されます。

### 強調表現

LukiWiki独自の視覚的強調：

```
''太字'' → <b>太字</b>
'''斜体''' → <i>斜体</i>
```

Markdownのセマンティック強調も利用可能：

```
**強調** → <strong>強調</strong>
*強調* → <em>強調</em>
```

### 取り消し線

2種類の取り消し線構文をサポート：

```
%%LukiWiki取り消し線%% → <s>LukiWiki取り消し線</s>
~~GFM取り消し線~~ → <del>GFM取り消し線</del>
```

- `%%text%%` - LukiWiki形式（`<s>`タグ）
- `~~text~~` - GitHub Flavored Markdown形式（`<del>`タグ）

両方の構文を同じドキュメント内で使い分けることができます。

### ブロック装飾

行頭にプレフィックスを配置：

```
COLOR(red): 赤い文字
SIZE(1.5): 大きな文字
RIGHT: 右寄せ
CENTER: 中央寄せ
```

### ブロック引用

LukiWiki形式（開始・終了タグ）：

```
> 引用文
> 複数行対応 <
```

Markdown形式（行頭プレフィックス）も使用可能：

```
> Markdownスタイルの引用
```

## ビルド

### 通常ビルド

```bash
cargo build --release
```

### WASM ビルド

```bash
wasm-pack build --target web
```

## テスト

```bash
cargo test
```

**テスト結果**: 112 tests passing

- 72 unit tests (including 5 frontmatter + 3 custom header ID tests)
- 18 CommonMark compliance tests
- 13 conflict resolution tests
- 9 doctests

## ライセンス

MIT License

## 作者

Masashi Yoshikawa

## リポジトリ

https://github.com/logue/LukiWiki-rs
