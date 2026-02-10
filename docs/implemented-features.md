# 実装済み機能リファレンス

**最終更新**: 2026年2月9日

このドキュメントはUniversal Markdownで実装済みの機能を記載しています。

## 目次

- [基本Markdown機能](#基本markdown機能)
- [コメント構文](#コメント構文)
- [メディアファイル自動検出](#メディアファイル自動検出)
- [Spoiler機能](#spoiler機能)
- [定義リスト](#定義リスト)
- [UMD拡張構文](#umd拡張構文)
- [プラグインシステム](#プラグインシステム)
  - [標準プラグイン](#標準プラグイン)
- [テーブル機能](#テーブル機能)
- [その他の機能](#その他の機能)

---

## 基本Markdown機能

### 見出し

ATX形式の見出しをサポート:

```markdown
# 見出し1

## 見出し2

### 見出し3

#### 見出し4

##### 見出し5

###### 見出し6
```

### 段落と改行

段落は空行で区切ります。行末に2つのスペースまたは`\`を追加することで改行できます。

### フェンスコードブロック

````markdown
```rust
fn main() {
    println!("Hello, world!");
}
```
````

### リスト

**順序なしリスト**:

```markdown
- アイテム1
- アイテム2
- アイテム3
```

**注意**: `* アイテム` 構文もCommonMark標準でサポートされていますが、多くのエディタで `- アイテム` に自動整形されるため、`-` の使用を推奨します。

**順序付きリスト**:

```markdown
1. 最初
2. 次
3. さらに
```

### リンクと画像

**通常のリンク**:

```markdown
[リンクテキスト](https://example.com)
![画像の代替テキスト](image.png)
```

**明示的なURL自動リンク**:

```markdown
<https://example.com>
<mailto:user@example.com>
```

出力: `<a href="https://example.com">https://example.com</a>`

**注意**: 裸のURL（`http://example.com`）は自動リンク化されません。`<URL>` 形式で明示的にマークアップする必要があります。これは誤検出を防ぎ、意図したリンクのみを有効化するための仕様です。

### 強調

```markdown
_斜体_
**太字**
```

**注意**:

- `*斜体*` 構文もCommonMark標準でサポートされていますが、多くのエディタで `_斜体_` に自動整形されるため、`_` の使用を推奨します。
- `__text__` はCommonMark標準では `<strong>` ですが、Universal Markdownでは Discord風に `<u>` (アンダーライン) に変換されます。

```umd
__アンダーライン__ → <u>アンダーライン</u> (Discord風)
```

---

## UMD拡張構文

### 強調構文

**UMD形式の強調** (視覚的な装飾):

```umd
''太字'' → <b>太字</b>
'''斜体''' → <i>斜体</i>
__アンダーライン__ → <u>アンダーライン</u> (Discord風)
```

**注**:

- Markdown形式の強調（`_`, `**`）はセマンティックタグ（`<em>`, `<strong>`）を生成
- UMD形式は視覚的タグ（`<i>`, `<b>`, `<u>`）を生成
- `__text__` は Discord風にアンダーライン（`<u>`）として扱われます（CommonMark標準の`<strong>`ではありません）
- `*斜体*` はCommonMark標準でサポートされますが、多くのエディタで `_斜体_` に自動整形されます

### 取り消し線

```markdown
%%取り消し線%% → <s>取り消し線</s> (UMD: 視覚的)
~~取り消し線~~ → <del>取り消し線</del> (GFM: セマンティック)
```

---

## コメント構文

**実装日**: 2026年2月4日

プログラミング言語と同様のコメント構文。レンダリングされない開発用コメントをドキュメント内に記載できます。

### 単一行コメント

`//` から行末までがコメントとして扱われます。

```umd
// この行は出力されません
これは表示される // ここは表示されない
```

**出力**:

```html
<p>これは表示される</p>
```

### 複数行コメント

`/*` から `*/` までがコメントとして扱われます。

```umd
これは表示される
/*
このブロック内は
すべてコメント
*/
これも表示される
```

**出力**:

```html
<p>これは表示される</p>
<p>これも表示される</p>
```

### インラインコメント

行の途中にもコメントを配置できます。

```umd
前部分/* コメント */後部分
```

**出力**:

```html
<p>前部分後部分</p>
```

### コードブロック内での扱い

コードブロック（` ``` `）、インラインコード（`` ` ``）内のコメント構文は**そのまま保持**されます。

````markdown
```rust
// Rustのコメント
fn main() {}
```
````

**出力**:

```html
<pre><code class="language-rust">// Rustのコメント
fn main() {}
</code></pre>
```

### URLとの区別

URLスキーム（`https://`等）の `//` はコメントとして扱われません。

```markdown
リンク: https://example.com/path
```

**出力**: URLはそのまま保持されます。

### 処理タイミング

コメント除去は**Conflict Resolver段階**（Markdown解析前）で実行されます。

### 利点

1. **開発用メモ**: ドキュメント内にTODOや注意事項を記載
2. **一時的な無効化**: 構文を削除せずにコメントアウトで無効化
3. **コード例の説明**: コメント構文を使って例示の説明を追加

---

## メディアファイル自動検出

画像リンク構文 `![alt](url)` を拡張し、URLの拡張子に基づいて自動的に適切なHTML5メディアタグまたはダウンロードリンクに変換します。

### 動画ファイル

**対応拡張子**: `.mp4`, `.webm`, `.ogv`, `.mov`, `.avi`, `.mkv`, `.m4v` (大文字小文字区別なし)

**入力**:

```markdown
![Product demo](video.mp4 "製品デモ")
```

**出力**:

```html
<video controls title="製品デモ">
  <source src="video.mp4" type="video/mp4" />
  <track kind="captions" label="Product demo" />
  <a href="video.mp4" download class="download-link video-fallback"></a>
    🎬 Product demo
  </a>
</video>
```

**特徴**:

- ブラウザが`<video>`タグをサポートしていない場合、🎬アイコン付きダウンロードリンクが表示されます
- `download`属性により、ブラウザはファイルをダウンロードダイアログで開きます
- `class="download-link video-fallback"`でCSS/JSカスタマイズが可能

### 音声ファイル

**対応拡張子**: `.mp3`, `.wav`, `.ogg`, `.oga`, `.m4a`, `.aac`, `.flac`, `.opus`, `.weba` (大文字小文字区別なし)

**入力**:

```markdown
![Theme song](audio.mp3 "テーマソング")
```

**出力**:

```html
<audio controls title="テーマソング">
  <source src="audio.mp3" type="audio/mpeg" />
  <a href="audio.mp3" download class="download-link audio-fallback">
    🎵 Theme song
  </a>
</audio>
```

**特徴**:

- ブラウザが`<audio>`タグをサポートしていない場合、🎵アイコン付きダウンロードリンクが表示されます
- フォールバックにより古いブラウザでもファイルにアクセス可能

### 画像ファイル

**対応拡張子**: `.jpg`, `.jpeg`, `.png`, `.gif`, `.svg`, `.webp`, `.avif`, `.bmp`, `.ico`, `.jxl`

**入力**:

```markdown
![Company logo](logo.png "会社ロゴ")
```

**出力**:

```html
<picture title="会社ロゴ">
  <source srcset="logo.png" type="image/png" />
  <img src="logo.png" alt="Company logo" title="会社ロゴ" loading="lazy" />
</picture>
```

**特徴**:

- `<picture>`タグで最適な画像フォーマットをブラウザが選択
- `<img>`タグがフォールバックとして機能（HTML5標準）

### ダウンロード可能ファイル

メディアとして認識されないファイルは、📄アイコン付きダウンロードリンクに変換されます。

**対応拡張子**:

- **アーカイブ**: `.zip`, `.tar`, `.gz`, `.7z`, `.rar`, `.bz2`, `.xz`
- **ドキュメント**: `.pdf`, `.doc`, `.docx`, `.xls`, `.xlsx`, `.ppt`, `.pptx`, `.odt`, `.ods`, `.odp`
- **テキスト**: `.txt`, `.md`, `.csv`, `.json`, `.xml`, `.yaml`, `.yml`, `.toml`
- **実行ファイル**: `.exe`, `.dmg`, `.deb`, `.rpm`, `.app`, `.apk`, `.msi`

**入力**:

```markdown
![User Manual](manual.pdf "完全なユーザーガイド")
![Source Code](project.zip)
```

**出力**:

```html
<a
  href="manual.pdf"
  download
  class="download-link"
  title="完全なユーザーガイド"
>
  📄 User Manual
</a>

<a href="project.zip" download class="download-link">📄 Source Code</a>
```

**特徴**:

- 📄絵文字でファイルダウンロードを視覚的に表現（TwiEmoji推奨）
- `download`属性でブラウザにダウンロード動作を指示
- `class="download-link"`でスタイリングのカスタマイズが可能
- altテキストが空の場合、URLをフォールバック表示

**セキュリティ注意事項**:

- 実行ファイル（`.exe`, `.dmg`等）もダウンロードリンクとして処理されます
- サーバー側で適切なContent-Typeヘッダを設定することを推奨します
- ユーザーからのファイルアップロードを許可する場合は、追加のセキュリティ対策が必要です

### 絵文字アイコン

各ファイルタイプに適した絵文字を使用：

- 🎬 (U+1F3AC) - 動画ファイル
- 🎵 (U+1F3B5) - 音声ファイル
- 🖼️ (U+1F5BC) - 画像ファイル（フォールバック内では未使用）
- 📄 (U+1F4C4) - 一般ファイル

**TwiEmoji推奨**: これらの絵文字をSVGアイコンとして美しく表示するため、[TwiEmoji](https://github.com/twitter/twemoji)の使用を推奨します。

### 共通機能

- **自動検出**: 拡張子に基づいて適切なHTML要素を自動生成
- **title属性対応**: CommonMark標準のタイトル属性 (`![alt](url "title")`) に対応
- **HTML5準拠**: 最新のHTML5標準に準拠
- **遅延読み込み**: 画像には `loading="lazy"` 属性を自動追加
- **アクセシビリティ**: 動画のaltテキストはキャプションラベルとして使用
- **フォールバック**: 非対応ブラウザでもコンテンツにアクセス可能

### 使用例

```markdown
# プロジェクト紹介

製品デモ動画:

![製品デモ](demo.mp4 "当社の製品をご紹介します")

テーマソング:

![BGM](theme.mp3 "オープニングテーマ")

最新の画像フォーマット:

![次世代画像](modern.jxl "JPEG XL形式")
```

---

## Spoiler機能

Discord風のSpoiler構文をサポートしています。

### Discord構文

```umd
このキャラは||実は悪役||だった。
```

### UMD装飾関数形式

```umd
このキャラは&spoiler{実は悪役};だった。
```

### 出力

```html
このキャラは<span
  class="spoiler"
  role="button"
  tabindex="0"
  aria-expanded="false"
  >実は悪役</span
>だった。
```

デフォルトではコンテンツが隠され、クリックまたはタップで表示されます。アクセシビリティのため、`role="button"`、`tabindex="0"`、`aria-expanded`属性が自動的に追加されます。

### CSSスタイリング（推奨）

```css
.spoiler {
  background-color: #202225;
  color: transparent;
  cursor: pointer;
  transition: color 0.1s ease;
}

.spoiler:hover,
.spoiler:active,
.spoiler.revealed {
  background-color: #2f3136;
  color: inherit;
}
```

### JavaScript実装（オプション）

```javascript
document.querySelectorAll(".spoiler").forEach((el) => {
  const toggle = () => {
    const isRevealed = el.classList.toggle("revealed");
    el.setAttribute("aria-expanded", isRevealed);
  };
  el.addEventListener("click", toggle);
  el.addEventListener("keydown", (e) => {
    if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      toggle();
    }
  });
});
```

---

## 定義リスト

用語と定義をセマンティックにマークアップできます。

### 構文

```umd
:HTML|HyperText Markup Language
:CSS|Cascading Style Sheets
:JavaScript|プログラミング言語
```

### 出力

```html
<dl>
  <dt>HTML</dt>
  <dd>HyperText Markup Language</dd>
  <dt>CSS</dt>
  <dd>Cascading Style Sheets</dd>
  <dt>JavaScript</dt>
  <dd>プログラミング言語</dd>
</dl>
```

### 複数定義

同じ用語に複数の定義を持たせることができます:

```umd
:用語1|定義1-1
:用語1|定義1-2
```

出力:

```html
<dl>
  <dt>用語1</dt>
  <dd>定義1-1</dd>
  <dd>定義1-2</dd>
</dl>
```

### ブロック要素を含む定義

`|` の後に改行を入れることで、定義部分にテーブルやリストなどのブロック要素を含めることができます。

**テーブルを含む例**:

```umd
:タイトル1|
  | ヘッダー1 | ヘッダー2 |
  | --------- | --------- |
  | アイテム1 | アイテム2 |
```

出力:

```html
<dl>
  <dt>タイトル1</dt>
  <dd>
    <table class="table">
      <thead>
        <tr>
          <th>ヘッダー1</th>
          <th>ヘッダー2</th>
        </tr>
      </thead>
      <tbody>
        <tr>
          <td>アイテム1</td>
          <td>アイテム2</td>
        </tr>
      </tbody>
    </table>
  </dd>
</dl>
```

**リストを含む例**:

```umd
:タイトル2|
  - リスト1
  - リスト2
  - リスト3
```

出力:

```html
<dl>
  <dt>タイトル2</dt>
  <dd>
    <ul>
      <li>リスト1</li>
      <li>リスト2</li>
      <li>リスト3</li>
    </ul>
  </dd>
</dl>
```

**仕様**:

- `|` の後に改行がある場合、次の行から次の `:` までが定義部分として扱われます
- **インデント（スペース2個）を推奨**: 可読性を考慮し、定義部分の各行にインデントを入れることを推奨します（必須ではありません）
- ブロック要素（テーブル、リスト、コードブロック等）が `<dd>` 内に配置されます
- この機能はPukiWiki/LukiWikiの類似機能にインスパイアされていますが、インデントの推奨など、一部仕様が異なります

### 使用例

- 用語集
- FAQ（よくある質問）
- 仕様書の定義
- 技術用語の説明

---

## ブロック引用

**UMD形式** (開始・終了タグ):

```umd
> 引用文
> 複数行の引用
> <
```

**Markdown標準形式**:

```markdown
> 引用文
> 複数行の引用
```

### ブロック装飾プレフィックス

行頭に配置して段落やブロックをスタイリング:

#### 配置

```umd
LEFT: 左寄せテキスト → <p class="text-start">...</p>
CENTER: 中央寄せテキスト → <p class="text-center">...</p>
RIGHT: 右寄せテキスト → <p class="text-end">...</p>
```

#### 色指定

```umd
COLOR(primary): 青いテキスト → Bootstrap text-primary
COLOR(,warning): 黄色の背景 → Bootstrap bg-warning
COLOR(danger,danger-subtle): 赤い強調テキスト → text-danger + bg-danger-subtle
COLOR(#FF0000): カスタム色 → インラインスタイル
```

**サポートされるBootstrap色**:

- テーマカラー: `primary`, `secondary`, `success`, `danger`, `warning`, `info`, `light`, `dark`
- グレースケール: `gray-100`～`gray-900`
- サブトル色: `primary-subtle`, `success-border-subtle`等

#### サイズ指定

```umd
SIZE(2): 大きいテキスト → <p class="fs-2">...</p>
SIZE(1.5rem): カスタムサイズ → <p style="font-size: 1.5rem">...</p>
```

**Bootstrapクラスマッピング**:

- `2.5` → `fs-1` (2.5rem)
- `2` → `fs-2` (2rem)
- `1.75` → `fs-3`
- `1.5` → `fs-4`
- `1.25` → `fs-5`
- `0.875` → `fs-6`

#### 複合構文

複数のプレフィックスを組み合わせ可能:

```umd
SIZE(1.5): COLOR(primary): CENTER: 強調テキスト
```

出力: `<p class="fs-4 text-primary text-center">強調テキスト</p>`

### インライン装飾関数

段落内でテキストをスタイリング:

#### 基本装飾

```umd
&color(danger){エラー}; → <span class="text-danger">エラー</span>
&size(2){大きい文字}; → <span class="fs-2">大きい文字</span>
&badge(success){Active}; → <span class="badge bg-success">Active</span>
&badge(primary){[New](/new)}; → <a href="/new" class="badge bg-primary">New</a>
```

#### セマンティック要素

```umd
&sup(上付き); → <sup>上付き</sup>
&sub(下付き); → <sub>下付き</sub>
&lang(en){Hello}; → <span lang="en">Hello</span>
&abbr(HTML){HyperText Markup Language}; → <abbr title="HyperText...">HTML</abbr>
&ruby(あした){明日}; → <ruby>明日<rp>(</rp><rt>あした</rt><rp>)</rp></ruby>
&kbd(Ctrl); → <kbd>Ctrl</kbd>
&samp(output); → <samp>output</samp>
&var(x); → <var>x</var>
&cite(Book Title); → <cite>Book Title</cite>
&q(Quote); → <q>Quote</q>
&small(Small text); → <small>Small text</small>
&u(Underline); → <u>Underline</u>
&time(2026-01-26){今日}; → <time datetime="2026-01-26">今日</time>
&dfn(Definition); → <dfn>Definition</dfn>
&bdi(Text); → <bdi>Text</bdi>
&bdo(rtl){Text}; → <bdo dir="rtl">Text</bdo>
&wbr; → <wbr />
&br; → <br />
```

**注意**: `&br;`は主にテーブルセル内での明示的な改行に使用します。通常の段落では行末に2つのスペースまたは`\`を使用してください。

---

## プラグインシステム

Universal Markdownは拡張可能なプラグインシステムを提供します。

### プラグイン構文

#### インライン型

```umd
&function(arg1,arg2){content}; 完全形
&function(args); 引数のみ
&function; 引数なし
```

#### ブロック型

```umd
@function(args){{ multi-line content }} 複数行
@function(args){single-line content} 単行
@function(args) 引数のみ
@function() 引数なし
```

### HTML出力形式

プラグインは`<template>`タグと`<data>`要素で出力されます:

**複数引数の例**:

```html
<template class="umd-plugin umd-plugin-function">
  <data value="0">arg1</data>
  <data value="1">arg2</data>
  content
</template>
```

- `class="umd-plugin umd-plugin-{関数名}"` - プラグイン識別用のクラス
- `<data value="インデックス">引数</data>` - 各引数は個別の`<data>`要素として格納（カンマ区切り）
- コンテンツはHTMLエスケープされてテキストノードとして保持（`&` → `&amp;`、`<` → `&lt;`など）
- ブロック型プラグインは前後に改行が入り、インライン型はインラインで出力されます

**再パース対応**:

コンテンツ内のWiki構文はエスケープされて保持されるため、バックエンド側で`<template>`要素のテキストコンテンツを取得し、再度パーサーに渡すことでネストされた構文も処理可能です：

```umd
@box(){{ **bold** and *italic* }}
```

↓

```html
<template class="umd-plugin umd-plugin-box">**bold** and *italic*</template>
```

バックエンド（Nuxt/Laravel等）でこのHTMLをパースし、最終的なHTML出力を生成します。

### 標準プラグイン

#### @detail - 詳細情報の折りたたみ

標準HTML `<details>` 要素を使った折りたたみコンテンツを生成します。

**構文**:

```umd
@detail(サマリーテキスト){{
  詳細な内容
}}

@detail(サマリーテキスト, open){{
  最初から開いている内容
}}
```

**パラメータ**:

- 第1引数: `<summary>` タグに表示されるテキスト（必須）
- 第2引数: `open` を指定すると初期状態で開いた状態になる（オプション）

**出力HTML**:

```html
<details>
  <summary>サマリーテキスト</summary>
  詳細な内容
</details>

<!-- open指定時 -->
<details open>
  <summary>サマリーテキスト</summary>
  最初から開いている内容
</details>
```

**特徴**:

- 標準HTMLのみを使用（JavaScriptやBootstrap不要）
- Bootstrapのアコーディオンと似た機能だが、よりシンプルで軽量
- ネストされた構文も再パース可能
- モバイルフレンドリー

**使用例**:

```umd
@detail(詳細を表示){{
  この内容は折りたたまれています。
  クリックすると表示されます。
}}

@detail(重要な情報, open){{
  この内容は最初から表示されています。
}}
```

---

## テーブル機能

### Markdown標準テーブル

GFM準拠のテーブル（ソート可能）:

```markdown
| Header1 | Header2 |
| ------- | ------- |
| Cell1   | Cell2   |
```

出力: `<table class="table">...</table>`

### UMDテーブル

セル連結対応の拡張テーブル:

```umd
| Header1 |>      | Header3 |
| Cell1   | Cell2 | Cell3   |
```

出力: `<table class="table umd-table">...</table>`

#### セル連結

**横方向連結 (colspan)**: `|>`

```umd
| Header1 |>      |
| Cell1   | Cell2 |
```

**縦方向連結 (rowspan)**: `|^`

```umd
| Header1 | Header2 |
| Cell1   | Cell2   |
|^        | Cell3   |
```

#### セル装飾

セル内でブロック装飾プレフィックスを使用可能:

```umd
| RIGHT: 右寄せ            | CENTER: 中央          |
| TOP: 上揃え              | MIDDLE: 中央揃え      |
| COLOR(primary): 青い文字 | SIZE(1.5): 大きい文字 |
```

---

## その他の機能

### フロントマター

YAMLまたはTOML形式のメタデータをサポート:

**YAML形式**:

```yaml
---
title: ページタイトル
author: 著者名
date: 2026-01-26
description: ページの説明文
tags:
  - universal markdown
  - umd
globs:
  - "**/api/**/*.umd"
priority: 2
alwaysApply: false
---
```

**TOML形式**:

```toml
+++
title = "ページタイトル"
author = "著者名"
date = 2026-01-26
description = "ページの説明文"
tags = ["universal markdown", "umd"]
globs = ["**/api/**/*.umd"]
priority = 2
alwaysApply = false
+++
```

フロントマターはHTML出力から除外され、`ParseResult.frontmatter`で取得できます。

### フットノート

Markdown標準の脚注構文:

```markdown
本文中の参照[^1]です。

[^1]: 脚注の内容
```

**出力形式**:

フットノートはHTML化されず、本文と分離して構造化データとして`ParseResult.footnotes`で取得できます。

```rust
pub struct ParseResult {
    pub html: String,
    pub frontmatter: Option<Frontmatter>,
    pub footnotes: Option<Vec<Footnote>>,
}

pub struct Footnote {
    pub id: String,      // 例: "1", "note-name"
    pub content: String, // Markdown形式の内容
}
```

**使用例**:

```rust
let result = parse_with_frontmatter(input);

// 本文HTML
println!("HTML: {}", result.html);

// フットノートデータ
if let Some(footnotes) = result.footnotes {
    for footnote in footnotes {
        println!("Footnote [^{}]: {}", footnote.id, footnote.content);
    }
}
```

**JSON出力例**:

```json
[
  {
    "id": "1",
    "content": "脚注の内容"
  },
  {
    "id": "note-name",
    "content": "名前付き脚注の内容"
  }
]
```

**HTML化**:

フットノートのHTML化はバックエンド（Nuxt/Laravel等）側で処理します。これにより、アプリケーションごとに異なるスタイリングやレイアウトを柔軟に適用できます。

**ネストされたフットノート**:

フットノートの内容内にさらにフットノート参照（`[^n]`）が含まれている場合、それはMarkdownテキストとしてそのまま保持されます。

```markdown
[^1]: 最初のフットノート[^2]を含む

[^2]: ネストされたフットノート
```

この場合の出力:

```json
[
  {
    "id": "1",
    "content": "最初のフットノート[^2]を含む"
  },
  {
    "id": "2",
    "content": "ネストされたフットノート"
  }
]
```

バックエンド側で再パースする際に、フットノート参照記号を適切に処理できます。深いネストは可読性を損なうため推奨されませんが、技術的には処理可能です。

**オプション設定**:

将来的に、`ParserOptions.render_footnotes_inline: bool` オプションで、フットノートをHTML化して本文に含めるモードも検討中です。

### カスタムヘッダーID

見出しに任意のIDを指定:

```markdown
# カスタムID付き見出し {#custom-id}
```

出力: `<h1 id="custom-id">カスタムID付き見出し</h1>`

指定がない場合は自動で`heading-1`, `heading-2`...と採番されます。

### GFM Callouts (アラート)

GitHub Flavored Markdown互換のアラート:

```markdown
> [!NOTE]
> 補足情報

> [!TIP]
> 役立つヒント

> [!IMPORTANT]
> 重要な情報

> [!WARNING]
> 警告

> [!CAUTION]
> 注意事項
```

それぞれBootstrapのalertクラスに変換されます。

---

## セキュリティ

### HTML入力制限

- 直接のHTML入力は**完全禁止**
- すべてのHTMLタグは自動エスケープ
- HTMLエンティティ（`&nbsp;`, `&lt;`等）のみ保持
- パーサー生成のHTMLのみ出力に使用

### URL Sanitization

以下のスキームをブロック:

- `javascript:` - XSS攻撃
- `data:` - Base64エンコードスクリプト
- `vbscript:` - VBScript実行
- `file:` - ローカルファイルアクセス（デフォルト）

**注**: `file:`スキームはセキュリティ上の理由でデフォルトでブロックされますが、スタンドアロンアプリケーション（オフラインヘルプシステムなど）での使用を想定した設定オプションの追加を検討中です（[planned-features.md](planned-features.md)を参照）。

**ブロック時の動作**:

危険なスキームを含むURLは、リンクとして処理されず、プレーンテキストとしてそのまま出力されます。

```markdown
<data:text/html,test> → data:text/html,test (リンク化されない)
<javascript:alert(1)> → javascript:alert(1) (リンク化されない)
<file:///etc/passwd> → file:///etc/passwd (リンク化されない)
<https://example.com> → <a href="https://example.com">...</a> (正常にリンク化)
```

これは通常のリンク構文 `[text](url)` と明示的なautolink `<url>` の両方に適用されます。

**許可されるスキーム**:

- 標準プロトコル: `http:`, `https:`, `mailto:`, `tel:`, `ftp:`
- カスタムアプリスキーム: `spotify:`, `discord:`, `vscode:`, `steam:` など
- 相対パス: `/path`, `./path`, `#anchor`

---

## テスト結果

**総テスト数**: 184 tests passing

- 121 unit tests
- 22 bootstrap integration tests
- 18 CommonMark compliance tests
- 13 conflict resolution tests
- 10 other tests
