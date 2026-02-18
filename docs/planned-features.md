# 実装予定機能リファレンス

**最終更新**: 2026年2月9日

このドキュメントは実装予定だが、まだ実装されていない機能を記載しています。

## 目次

- [~~メディアファイル自動検出~~](#メディアファイル自動検出) ✅ **実装済み** (2026/02/09)
- [ブロック装飾の追加機能](#ブロック装飾の追加機能)
- [テーブル拡張](#テーブル拡張)
- [Markdown拡張機能](#markdown拡張機能)
- [セキュリティ設定オプション](#セキュリティ設定オプション提案)
- [高度なUMD機能](#高度なumd機能)
  - [数式サポート](#数式サポートmath-formula-support)
  - [ポップオーバー](#ポップオーバーpopover)
- [テンプレートエンジン機能（将来構想）](#テンプレートエンジン機能将来構想)
- [未実装機能（提案段階）](#未実装機能提案段階)
- [サポートしない機能](#サポートしない機能)
  - [絵文字ショートコード](#絵文字ショートコード)

---

## メディアファイル自動検出

> ✅ **実装済み** (2026年2月9日)  
> この機能は実装が完了しました。詳細は [docs/implemented-features.md#メディアファイル自動検出](implemented-features.md#メディアファイル自動検出) を参照してください。

### 概要

画像リンク構文 `![alt](url)` を拡張し、URLの拡張子に基づいて自動的に適切なHTMLメディアタグに変換します。

### 設計方針: HTML5メディアタグ統一

- 全てのメディアを最新のHTML5タグで処理（古いブラウザは考慮しない）
- 画像も`<picture>`タグで出力し、将来的な拡張性を確保
- `<video>`, `<audio>`, `<picture>`で統一的なメディア処理を実現

### 動画ファイル

**対応拡張子** (大文字小文字区別なし):
`.mp4`, `.webm`, `.ogv`, `.ogg`, `.mov`, `.avi`, `.mkv`, `.m4v`

**出力HTML**:

```html
<video controls title="title">
  <source src="url" type="video/ext" />
  <track kind="captions" label="alt" />
  お使いのブラウザは動画タグをサポートしていません。
</video>
```

**特徴**:

- `<source>`タグで明示的にMIMEタイプを指定
- `alt`テキストは`<track>`タグのキャプションラベルとして使用
- `controls`属性をデフォルトで追加
- CommonMark標準のタイトル属性（`![alt](url "title")`）に対応

**使用例**:

```markdown
![プレゼンテーション](video.mp4)
![製品デモ](demo.mp4 "新製品のデモンストレーション")
```

### 音声ファイル

**対応拡張子** (大文字小文字区別なし):
`.mp3`, `.wav`, `.ogg`, `.oga`, `.m4a`, `.aac`, `.flac`, `.opus`, `.weba`

**出力HTML**:

```html
<audio controls title="title">
  <source src="url" type="audio/ext" />
  お使いのブラウザは音声タグをサポートしていません。
</audio>
```

**特徴**:

- `<source>`タグで明示的にMIMEタイプを指定
- `controls`属性をデフォルトで追加
- CommonMark標準のタイトル属性（`![alt](url "title")`）に対応

**使用例**:

```markdown
![BGM](audio.mp3)
![テーマソング](theme.mp3 "オープニングテーマ")
```

### 画像ファイル

**対応拡張子**:
`.jpg`, `.jpeg`, `.png`, `.gif`, `.svg`, `.webp`, `.avif`, `.bmp`, `.ico` 等

**出力HTML**:

```html
<picture title="title">
  <source srcset="url" type="image/ext" />
  <img src="url" alt="alt" title="title" loading="lazy" />
</picture>
```

**特徴**:

- `<picture>`タグで統一的に出力
- `<source>`タグで明示的にMIMEタイプを指定
- CommonMark標準のタイトル属性（`![alt](url "title")`）を`<picture>`と`<img>`の両方に設定
- `<img>`タグはフォールバック兼アクセシビリティ対応
- `loading="lazy"`属性を自動追加（遅延読み込み）

**インライン表示への配慮**:

- `<picture>`タグは`inline`要素として扱われる
- 段落内での使用が可能: `テキスト![画像](image.png)テキスト`
- CSSでブロック化も可能: `.picture { display: block; }`

**使用例**:

```markdown
![ロゴ](logo.png)
![最新画像](image.avif)
```

### MIMEタイプマッピング

#### 動画

- mp4→video/mp4, webm→video/webm, ogv→video/ogg, ogg→video/ogg
- mov→video/quicktime, avi→video/x-msvideo, mkv→video/x-matroska, m4v→video/x-m4v

#### 音声

- mp3→audio/mpeg, wav→audio/wav, ogg→audio/ogg, oga→audio/ogg
- m4a→audio/mp4, aac→audio/aac, flac→audio/flac, opus→audio/opus, weba→audio/webm

#### 画像

- jpg/jpeg→image/jpeg, png→image/png, gif→image/gif, svg→image/svg+xml
- webp→image/webp, avif→image/avif, bmp→image/bmp, ico→image/x-icon

### 実装方針

1. comrakのAST後処理で`Image`ノードを検出
2. URLの拡張子を正規表現で解析（クエリパラメータを除外）
3. 拡張子に応じて適切なHTMLタグとMIMEタイプを生成
4. 拡張子なし、または未知の拡張子: 画像として扱う（`<picture>`+`image/octet-stream`）

### セキュリティ

URL sanitizationは既存のsanitizer.rsで処理。以下のスキームをブロック:

- `javascript:` - JavaScript実行
- `data:` - Base64エンコードされたスクリプト
- `vbscript:` - VBScript実行
- `file:` - ローカルファイルアクセス（デフォルト）

**注**: `file:`スキームはセキュリティ上の理由でデフォルトでブロックされますが、スタンドアロンアプリケーション（オフラインヘルプシステムなど）のユースケースでは有用です。グローバルコンフィグで許可できる仕様を検討中（後述の「セキュリティ設定オプション」を参照）。

### 拡張オプション（将来的な検討事項）

`@media`プラグインで追加属性・複数ソースを指定可能にする:

**ブロック型**:

```umd
@media(autoplay,loop,muted){![動画](video.mp4)}
@media(preload=metadata){![音声](audio.mp3)}
@media(mobile.jpg,tablet.jpg,desktop.jpg){![レスポンシブ](desktop.jpg)}
@media(avif,webp,fallback.png){![最新画像](image.avif)}
```

**インライン型**:

```umd
テキスト&media(width=100){![アイコン](icon.png)};テキスト
説明文&media(autoplay,muted){![デモ](demo.mp4)};説明文
```

#### 行の先頭メディアの自動ブロック化

行の先頭に配置されたメディアタグ（`![alt](url)`）を自動的にブロック要素として扱う拡張を検討。

**設計方針**:

- 段落の先頭または独立した行にある画像をブロックとして扱う
- `<figure>`タグで囲んでセマンティックな構造を提供
- `<figcaption>`でaltテキストをキャプションとして表示
- インライン使用時は従来通り`<picture>`タグを使用

**出力HTML例**:

```html
<figure class="umd-media-block">
  <picture title="title">
    <source srcset="url" type="image/ext" />
    <img src="url" alt="alt" title="title" loading="lazy" />
  </picture>
  <figcaption>alt</figcaption>
</figure>
```

**使用例**:

```markdown
# ドキュメント

![メイン画像](hero.jpg "サイトのヒーロー画像")

本文テキスト...

CENTER:
![中央寄せ画像](centered.png "中央に配置される画像")

![スクリーンショット](screenshot.png)
```

**利点**:

- `CENTER:`プレフィックスとの組み合わせで、画像の中央寄せが可能
- ブロック要素として扱われるため、他の配置プレフィックス（`LEFT:`, `RIGHT:`, `JUSTIFY:`）も適用可能
- セマンティックな`<figure>`構造でアクセシビリティ向上

**実装検討**:

- AST解析で画像ノードの位置を検出
- ブロック先頭判定: 段落の最初の子、または空行後の独立行
- 既存のインライン動作を維持しつつ、ブロック時のみ拡張適用

---

## ブロック装飾の追加機能

### JUSTIFY プレフィックス

#### テキストの両端揃え

```umd
JUSTIFY: この文章は両端揃えで表示されます。
```

出力: `<p class="text-justify">この文章は両端揃えで表示されます。</p>`

**注**: Bootstrap 5では`text-justify`が非推奨だが、UMDでは明示的に対応。

#### UMDテーブルの幅指定

UMDテーブル（区切り行なし）の前に`JUSTIFY:`がある場合、テーブル全体の幅を100%に維持:

```umd
JUSTIFY:
| Header1 | Header2 |
| Cell1   | Cell2   |
```

出力: `<table class="table umd-table">...</table>`

### UMDブロック要素の配置まとめ

適用対象: UMDテーブル（区切り行なし）、ブロック型プラグイン（`@function(...)`）

- `LEFT:`（改行）`<block>` → `w-auto`（コンテンツ幅、左寄せ）
- `CENTER:`（改行）`<block>` → `w-auto mx-auto`（コンテンツ幅、中央寄せ）
- `RIGHT:`（改行）`<block>` → `w-auto ms-auto me-0`（コンテンツ幅、右寄せ）
- `JUSTIFY:`（改行）`<block>` → `w-100`（100%幅）

**例（ブロック型プラグイン）**:

```umd
CENTER:
@callout(info)
```

出力: `<div class="umd-plugin umd-plugin-callout w-auto mx-auto" ...></div>`

### TRUNCATE プレフィックス

テキスト省略:

```umd
TRUNCATE: 長いテキストは省略されます...
```

出力: `<p class="text-truncate">長いテキストは省略されます...</p>`

**特徴**:

- `overflow: hidden; text-overflow: ellipsis; white-space: nowrap`を適用
- 幅指定はユーザーがCSSで指定する前提
- テーブルセルでは自動適用される

---

## テーブル拡張

### テーブルバリエーション

プラグインシステムで実装予定:

```umd
@table(striped,hover){{
  | Header1 | Header2 |
  | Cell1   | Cell2   |
}}
```

**インデントの推奨**:

プラグインの `{{}}` 内のコンテンツは、可読性を向上させるため**2スペースのインデント**を推奨します：

```umd
@table(striped){{
  | Header1 | Header2 | Header3 |
  |---------|---------|---------|
  | Cell1   | Cell2   | Cell3   |
  | Data1   | Data2   | Data3   |
}}
```

インデントは構文上必須ではありませんが、以下の利点があります：

- プラグインの範囲が視覚的に明確になる
- ネストされたコンテンツの構造が理解しやすい
- コードレビューやメンテナンスが容易になる

**サポート予定のオプション**:

- `striped` → `table-striped`（縞模様）
- `hover` → `table-hover`（ホバー効果）
- `dark` → `table-dark`（ダークモード）
- `bordered` → `table-bordered`（ボーダー）
- `borderless` → `table-borderless`（ボーダーなし）
- `sm` → `table-sm`（コンパクト）

### レスポンシブ対応

```umd
@table(responsive){{
  | Header1 | Header2 |
  | Cell1   | Cell2   |
}}
```

出力: `<div class="table-responsive"><table>...</table></div>`

### 仕様と制約

**処理対象**:

- `{{}}` 内の**最初の1つのテーブルのみ**にスタイルを適用
- GFM形式・UMD形式の両方に対応

**複数テーブルの扱い**:

```umd
@table(hover){{
  | Table 1 | A |

  | Table 2 | B |
}}
```

→ 最初のテーブル（Table 1）のみに`table-hover`が適用される

**テーブルが存在しない場合**:

```umd
@table(striped){{
  これはテキストです
}}
```

→ 警告を出力し、コンテンツはそのまま表示

**ネストされた@tableプラグイン**:

```umd
@table(hover){{
  | foo | bar |
  | --- | ---:|
  | 1   |   2 |
  @table(striped){{
    | key   | value |
    | alpha | one   |
  }}
}}
```

→ **非推奨**: 外側のテーブル内にネストされた`@table`プラグインは予期しない動作を引き起こす可能性があります。各テーブルは個別の`@table`ブロックで指定してください。

**推奨される書き方**:

```umd
@table(hover){{
  | foo | bar |
  | --- | ---:|
  | 1   |   2 |
}}

@table(striped){{
  | key   | value |
  | alpha | one   |
}}
```

### ブロック引用のデフォルトクラス

```html
<blockquote class="blockquote">...</blockquote>
```

Bootstrap標準の引用スタイルを自動適用。

---

## Markdown拡張機能

### Setext見出し

下線形式の見出し:

```markdown
# 見出し1

## 見出し2
```

### 参照スタイルリンク

```markdown
[テキスト][ref]

[ref]: https://example.com
```

### バックスラッシュエスケープ

```markdown
\*リテラルアスタリスク\*
```

### URL自動リンク（明示的）

```markdown
<https://example.com>
<mailto:user@example.com>
```

**方針**: 裸のURL（`http://example.com`）の自動リンク化は**行いません**。

**理由**:

1. **誤検出の防止**: 日本語などマルチバイト文字を含む文章では、URL直後のテキストがアドレスの一部として誤認されやすい
2. **文脈の保護**: 「`http://`で始まる」のような説明文が意図せずリンク化されるのを防ぐ
3. **明示性の向上**: `<URL>` 形式で明示的にマークアップすることで、意図したリンクのみが有効化される

**セキュリティ制約**:

危険なスキーム（`javascript:`, `data:`, `vbscript:`, `file:`）を含むURLは、リンクとして処理されず、プレーンテキストとして出力されます。

```markdown
<data:text/html,test> → data:text/html,test (リンク化されない)
<javascript:alert(1)> → javascript:alert(1) (リンク化されない)
<file:///etc/passwd> → file:///etc/passwd (リンク化されない)
<https://example.com> → <a href="...">https://example.com</a> (正常にリンク化)
```

これにより、XSS攻撃やローカルファイルアクセスなどのセキュリティリスクを防ぎます。

この仕様はCommonMark標準に準拠しています。

### コメント構文

Markdown文書内にコメントを記載するための構文を提供します。コメントはレンダリング結果に影響を与えません。

#### HTMLコメント形式 (`<!-- -->`)

オリジナルのMarkdownと同じように、HTMLコメントとして出力されます。

```markdown
これは表示される <!-- この部分はHTMLコメントとして出力される -->
```

**出力HTML**:

```html
<p>
  これは表示される
  <!-- この部分はHTMLコメントとして出力される -->
</p>
```

**特徴**:

- HTMLコメントとしてブラウザに残る
- 開発者ツールなどで確認可能
- ソースコードの可読性を保ちつつ、追加情報を記載

#### 処理しないコメント形式 (`/* ~ */`)

完全に処理されず、出力に含まれないコメントです。

```umd
これは表示される /* この部分は完全に除去される */
```

**出力HTML**:

```html
<p>これは表示される</p>
```

**特徴**:

- 出力HTMLから完全に除去
- 開発用メモや一時的な無効化に適する
- 最終出力のサイズを小さく保つ

#### コードブロック内での扱い

コードブロック（` ``` `）内の `/* ~ */` はそのまま出力され、コメントとして処理されません。

````markdown
```javascript
function example() {
  /* このコメントはコードの一部として保持される */
  return true;
}
```
````

````

**出力HTML**:

```html
<pre><code class="language-javascript">function example() {
  /* このコメントはコードの一部として保持される */
  return true;
}
</code></pre>
````

**意味の区別**:

- `<!-- -->`: HTMLソースに残るコメント（ドキュメントのメタ情報）
- `/* ~ */`: 完全に処理しないコメント（開発メモや無効化）

---

## セキュリティ設定オプション（提案）

### file:スキーマの条件付き許可

**背景**:

`file:`スキーマは通常、情報漏洩のリスクがあるためブロックされますが、特定のユースケースでは必要になります：

- スタンドアロンソフトウェアのオフラインヘルプシステム
- ローカルドキュメント管理アプリケーション
- Electron/Tauriアプリでのローカルリソースアクセス

**実装案**:

`ParserOptions`に設定オプションを追加：

```rust
pub struct ParserOptions {
    pub gfm_extensions: bool,
    // ... existing fields ...

    /// Allow file:// scheme in URLs (default: false)
    /// WARNING: Only enable in trusted, sandboxed environments
    pub allow_file_scheme: bool,
}
```

**使用例**:

```rust
// Webアプリケーション（デフォルト - file: ブロック）
let options = ParserOptions::default();

// スタンドアロンアプリ（file: 許可）
let options = ParserOptions {
    allow_file_scheme: true,
    ..ParserOptions::default()
};

let html = parse_to_html(markdown, &options);
```

**セキュリティ考慮事項**:

1. **デフォルトは無効**: Webアプリケーションの安全性を優先
2. **明示的な有効化**: 開発者が意図的に有効化する必要がある
3. **ドキュメント警告**: 有効化のリスクを明確に文書化
4. **サンドボックス推奨**: Electron等のサンドボックス環境での使用を推奨

**対象スキーム**:

- `file:` - 条件付き許可可能
- `javascript:`, `data:`, `vbscript:` - 常にブロック（設定不可）

---

## 高度なUMD機能

### リスト内ブロック要素

✅ **実装済み** (2026年2月18日)

リスト項目内にテーブル、コードブロック等を配置:

```markdown
- リスト項目
  | Header |
  | ------ |
  | Cell |
```

**出力HTML**:

```html
<ul>
  <li>
    <p>リスト項目</p>
    <table class="table">
      <thead>
        <tr>
          <th>Header</th>
        </tr>
      </thead>
      <tbody>
        <tr>
          <td>Cell</td>
        </tr>
      </tbody>
    </table>
  </li>
</ul>
```

**実装詳細**:

- [src/extensions/nested_blocks.rs](../src/extensions/nested_blocks.rs)で実装
- インデント解析による親子関係判定
- リスト項目直後のブロック要素（テーブル、コードブロック、ブロック引用、プラグイン、配置プレフィックス）を自動インデント
- 子要素は `<li>` タグ内に配置される
- CommonMark違反だが既存UMDコンテンツとの互換性のため必須

### タスクリスト拡張

✅ **実装済み** (2026年2月18日)

```umd
- [ ] 未完了タスク
- [x] 完了タスク
- [-] 不確定状態（UMD拡張）
```

**実装詳細**:

- [src/extensions/preprocessor.rs](../src/extensions/preprocessor.rs)でプレプロセス時に`[-]`をプレースホルダーに変換
- [src/extensions/conflict_resolver.rs](../src/extensions/conflict_resolver.rs)でポストプロセス時に`<input>`要素に`data-task="indeterminate"`と`aria-checked="mixed"`属性を追加
- comrak標準のタスクリスト（`[ ]`/`[x]`）を拡張
- 不確定状態はデータ属性とARIA属性で表現し、フロントエンドJavaScriptで動的に処理可能

### カスタムリンク属性

✅ **実装済み** (2026年2月18日)

リンクに`id`と`class`属性を追加できます。標準のMarkdown構文 `[Link text](URL "title")` でtitle属性にtitleが入ります。

**構文**:

```umd
[テキスト](url){id-name class1 class2}
[テキスト](url){#id-name}
[テキスト](url){.class1 .class2}
```

**実装詳細**:

- [src/extensions/conflict_resolver.rs](../src/extensions/conflict_resolver.rs)でポストプロセス時に実装
- 正規表現でリンク直後の`{...}`を検出
- 第一トークン（`#`プレフィックスなし）をIDとして優先的に設定
- `#id`形式で明示的にIDを指定可能
- `.class`形式で明示的にクラスを指定可能
- スペース区切りで複数クラスを指定可能
- 既存の`class`属性とマージして重複を回避

**使用例**:

```umd
[ドキュメント](docs){docs-link btn btn-primary}
[ホーム](/){\.nav-link active}
[GitHub](https://github.com){\.external target-blank}
```

**出力例**:

```html
<a href="docs" id="docs-link" class="btn btn-primary">ドキュメント</a>
<a href="/" class="nav-link active">ホーム</a>
<a href="https://github.com" class="external target-blank">GitHub</a>
```

### パス基準URL設定

パーサーオプションで `base_url` を指定することで、絶対パスをシステムの文脈に応じて動的に解決できます。

**設計背景**:

システムの絶対パス（例: `/path`）は、必ずしも独自ドメインのルートを指すわけではありません：

- **サブパスホスティング**: `https://example.com/umd-core/path`
- **複数アプリケーション**: `/app1/path` vs `/app2/path`
- **開発/本番環境の差異**: 開発時は `/dev` プレフィックス、本番時は `/prod` プレフィックス

**構文**:

パーサー初期化時にオプションで指定：

```rust
let options = ParserOptions {
    base_url: Some("/umd-core".to_string()),
    ..Default::default()
};

let result = parse_markdown(&input, &options);
```

**動作**:

- `base_url` が指定されている場合、Markdown内の絶対パス（`/` で始まるパス）を `base_url + path` に解決
- `base_url` が未指定の場合、パスはそのまま出力（従来動作）

**例**:

入力：

```markdown
[ドキュメント](/docs)
[API](/api/v1)
```

`base_url: "/umd-core"` の場合の出力：

```html
<a href="/umd-core/docs">ドキュメント</a> <a href="/umd-core/api/v1">API</a>
```

`base_url: "https://example.com/app"` の場合の出力：

```html
<a href="https://example.com/app/docs">ドキュメント</a>
<a href="https://example.com/app/api/v1">API</a>
```

**実装方針**:

- [src/lib.rs](../src/lib.rs) の `ParserOptions` に `base_url: Option<String>` フィールドを追加
- AST後処理時にリンク（`Node::Link`）とリソース（画像、動画等）の `url` フィールドを検査
- `/` で始まるパスに対して `base_url` を前置

**注意**:

- `base_url` の末尾に `/` がある場合は自動削除（`/umd-core/` → `/umd-core`）
- 相対パス（`./, ../path` など）への対応は検討の対象外（バックエンド側で処理推奨）
- プロトコル相対URL（`//example.com`）はそのまま出力
- 外部URL（`http://`, `https://` で始まる）はそのまま出力

### 数式サポート（Math Formula Support）

#### 概要

数式を表現するための明示的な構文を提供します。

#### 構文

```umd
&math(\sqrt{x^2});
&math(\frac{a}{b});
&math(\sum_{i=1}^{n} i);
&math(E = mc^2);
```

**設計方針**:

- LaTeX/KaTeX構文を使用
- セマンティック要素の構文規則に従い、パラメータなしの関数形式: `&math(式);`
- `$...$` 構文は**採用しません**

**理由**:

1. **金銭表記との競合**: 「The cost is $5.00.」のような文章で `$5.00` が数式デリミタとして誤認される可能性
2. **明示性の向上**: `&math(...);` で数式であることが明確
3. **エスケープの回避**: 金銭表記のたびに `\$` とエスケープするのは非効率的

#### 出力HTML

```html
<math xmlns="http://www.w3.org/1998/Math/MathML">
  <msqrt>
    <mi>x</mi>
    <mn>2</mn>
  </msqrt>
</math>
```

#### 実装方針

- パーサーがLaTeX式をMathMLに直接変換
- MathMLは標準的なXMLベースの数式表現形式（W3C仕様）
- ブラウザネイティブでサポートされており、追加ライブラリやバックエンド処理は不要
- LaTeX書式が標準化されているため、正確な変換が可能
- セマンティック要素の仕様により、インライン型 (`&math(...)`) とブロック型（プラグイン）の両方をサポート予定

### ポップオーバー（Popover）

#### 概要

HTML Popover APIを利用した軽量なポップアップコンテンツを提供します。トリガーとなるボタンをクリックすると、関連するコンテンツがポップオーバーとして表示されます。

**設計思想**:

- **インライン型** (`&`): テキスト中に埋め込み可能、単一行コンテンツのみ
- **ブロック型** (`@`): 独立したブロックとして配置、複数行コンテンツをサポート

この両方のバリエーションを提供することで、シンプルなケースから複雑なケースまで対応できます。

#### 構文

**インライン型**（単一行のみ）:

```umd
&popover(text){content};
```

**ブロック型**（複数行可能）:

```umd
@popover(text){{ content }}
@popover(text){content}
```

**パラメータ**:

- `text`: ポップオーバーを開くボタンのラベル
- `content`: ポップオーバー内に表示されるコンテンツ（Markdown記法可）

**使用例**:

インライン型（簡潔な説明）:

```umd
テキスト&popover(詳細){ 短い補足説明です };テキスト
```

ブロック型（複数行のコンテンツ）:

```umd
@popover(詳細を表示){{
  ここにポップオーバーの内容を記述します。

  - リスト項目1
  - リスト項目2
}}
```

#### 出力HTML

```html
<button command="show-popover" commandfor="umd-popover-a1b2c3d4">text</button>

<div id="umd-popover-a1b2c3d4" popover>contents</div>
```

**HTML要素の説明**:

- `<button>`: ポップオーバーを開くトリガー
  - `command="show-popover"`: Popover APIの標準コマンド
  - `commandfor="<id>"`: 関連するポップオーバー要素のIDを指定
- `<div popover>`: ポップオーバーコンテンツ
  - `id`: 一意な識別子（動的生成）
  - `popover`: ブラウザネイティブのPopover API属性

#### ID生成の仕組み

**ユニークID**:

ポップオーバー要素のIDは、`umd-popover-{UUID}`の形式で動的に生成されます：

- **プレフィックス**: `umd-popover-`（用途の明示）
- **UUID部分**: UUIDv4またはランダムな英数字（8文字以上）
- **例**: `umd-popover-a1b2c3d4`, `umd-popover-f9e8d7c6b5a4`

**生成タイミング**:

- パース時にRustのUUID生成ライブラリ（`uuid` crate）を使用
- 同一ドキュメント内で複数のポップオーバーが存在しても衝突しない

**実装例**:

```rust
use uuid::Uuid;

let popover_id = format!("umd-popover-{}", Uuid::new_v4().simple());
// => "umd-popover-a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6"
```

#### 今後の展開

この仕組みは、他のインタラクティブ要素でも共通して使用されます：

**ダイアログ（Dialog）**:

ブロック型（複数行可能）:

```umd
@dialog(開く){{
  # ダイアログタイトル

  ダイアログの本文コンテンツです。

  - リスト項目1
  - リスト項目2
}}
```

インライン型（単一行のみ）:

```umd
&dialog(開く){ ダイアログの内容 };
```

出力HTML:

```html
<button command="show-modal" commandfor="umd-dialog-x1y2z3">開く</button>

<dialog id="umd-dialog-x1y2z3">ダイアログの内容</dialog>
```

**ホバーツールチップ（Hover）**:

インライン型（単一行のみ）:

```umd
&hover(ツールチップの内容){ホバー対象テキスト};
```

ブロック型（複数行可能）:

```umd
@hover(ホバー対象テキスト){{
  **ツールチップタイトル**

  詳細な説明文
}}
```

出力HTML（提案）:

```html
<span aria-describedby="umd-hover-m1n2o3">ホバー対象テキスト</span>
<div role="tooltip" id="umd-hover-m1n2o3" hidden>ツールチップの内容</div>
```

**共通の設計方針**:

- **インライン型とブロック型の両方をサポート**
  - インライン型 (`&`): テキスト中に埋め込み、単一行コンテンツ
  - ブロック型 (`@`): 独立配置、複数行コンテンツ（`{{ }}` または `{}`）
- HTML標準APIの活用（`popover`, `dialog`, ARIA属性）
- ID生成の統一（`umd-{type}-{uuid}`）
- JavaScriptフレームワーク非依存
- アクセシビリティの考慮（`aria-*`, `role`属性）

#### ブラウザ対応

**Popover API**は以下のブラウザでサポートされています（2026年2月時点）:

- Chrome/Edge 114+
- Safari 17+
- Firefox 125+

古いブラウザでは、ポルフィル（polyfill）の使用が推奨されます。

---

## テンプレートエンジン機能（将来構想）

> 🚧 **将来構想**  
> この機能は正式バージョンリリース後の仕様策定段階です。現時点ではアイデアレベルであり、実装時期や詳細は未定です。

### 概要

フロントマターで定義した変数をMarkdown本文中で展開できるテンプレートエンジン機能を提供します。既存のプラグイン構文をマクロとして転用することで、統一的な記法を実現します。

### 設計方針

**責務の分離**:

- **UMDパーサー**: テンプレート構文を特殊なHTML要素として出力
- **バックエンド**: 実際の変数展開・マクロ実行を担当（Nuxt/Laravel/その他）

このアプローチにより、UMDパーサーは言語非依存のまま、各バックエンドが最適な方法でテンプレート処理を実装できます。

### フロントマター変数定義

```umd
---
title: "ページタイトル"
author: "著者名"
date: 2026-02-13
is_template: true  # テンプレートエンジン機能を有効化
config:
  theme: "dark"
  version: "1.0.0"
items:
  - "アイテム1"
  - "アイテム2"
  - "アイテム3"
---

# {{ title }}

著者: {{ author }}
日付: {{ date }}
```

**テンプレートモードの有効化**:

- `is_template: true` を設定することで、if文やfor文などの制御構造のサポートを有効化します。
- このフラグがない場合、基本的な変数展開のみが機能し、制御構造は無視されます。

### 変数展開構文

#### 基本的な変数展開

**構文**:

```umd
{{ variable }}
```

**UMDパーサー出力**:

```html
<data class="umd-var" value="variable">{{ variable }}</data>
```

または、フロントマターで定義された全変数を一覧として出力する場合：

```html
<datalist id="umd-frontmatter-vars">
  <option value="title">ページタイトル</option>
  <option value="author">著者名</option>
  <option value="date">2026-02-13</option>
</datalist>
```

**特徴**:

- 各変数展開箇所は `<data>` 要素で表現
- `value` 属性に変数名を格納
- 元の構文をテキストとして保持（バックエンドでの処理を想定）
- フロントマターのトップレベル変数を参照
- ドット記法でネストされた値にアクセス: `{{ config.theme }}`

#### 配列・オブジェクトのアクセス

```umd
{{ items.0 }}          <!-- 配列のインデックスアクセス -->
{{ config.theme }}     <!-- オブジェクトのプロパティアクセス -->
```

### マクロ機能（プラグイン構文の転用）

既存のプラグイン構文をマクロとして再利用します。

#### マクロ定義（フロントマター）

```yaml
---
macros:
  note: |
    @callout(info){{
      📝 **注意**: {content}
    }}
  button: |
    &link(btn btn-primary){{[{text}]({url})}}
---
```

#### マクロ呼び出し

```umd
@macro(note, content="これは重要な情報です");
@macro(button, text="クリック", url="/page");
```

**バックエンドでの展開イメージ**:

```umd
@callout(info){{
  📝 **注意**: これは重要な情報です
}}

&link(btn btn-primary){{[クリック](/page)}}
```

### 条件分岐（提案）

```umd
@if(theme == "dark"){{
  ダークモード用コンテンツ
}}

@if(version >= "1.0.0"){{
  新バージョンの機能説明
}}
```

**UMDパーサー出力**:

```html
<template class="umd-plugin umd-plugin-if" data-condition="theme == 'dark'">
  ダークモード用コンテンツ
</template>
```

### 繰り返し処理（提案）

```umd
@for(item in items){{
  - {{ item }}
}}
```

**期待される出力** (バックエンド処理後):

```markdown
- アイテム1
- アイテム2
- アイテム3
```

### セキュリティ考慮事項

**XSS対策**:

- 変数展開時は自動的にHTMLエスケープを適用
- 生のHTMLを挿入する場合は明示的なマーカーが必要（例: `{{{ raw_html }}}`）
- バックエンド側で適切なサニタイゼーションを実装

**インジェクション対策**:

- フロントマターのYAML/TOML解析は信頼できるライブラリを使用
- マクロ定義の再帰呼び出しを制限（無限ループ防止）
- 変数名の検証（予約語・特殊文字のチェック）

### 実装の段階的アプローチ

**Phase 1**: 基本変数展開

- フロントマター変数の定義
- `{{ variable }}` 構文のHTML要素化
- バックエンドでの単純な変数置換

**Phase 2**: マクロ機能

- `@macro()` 構文のサポート
- フロントマターでのマクロ定義
- プラグイン構文への展開

**Phase 3**: 制御構文

- `@if()`, `@for()` のサポート
- 条件評価エンジンの実装
- ネストされた制御構文

### Twigとの比較

| 機能       | Twig               | UMD提案            | 備考              |
| ---------- | ------------------ | ------------------ | ----------------- |
| 変数展開   | `{{ var }}`        | `{{ var }}`        | 同じ構文          |
| フィルター | `{{ var\|upper }}` | 未定               | バックエンド依存  |
| 条件分岐   | `{% if %}`         | `@if(){{}}`        | UMDプラグイン形式 |
| ループ     | `{% for %}`        | `@for(){{}}`       | UMDプラグイン形式 |
| マクロ     | `{% macro %}`      | フロントマター定義 | YAML/TOML形式     |
| 継承       | `{% extends %}`    | 未対応             | バックエンド責務  |

### 利用シーン

1. **ドキュメント生成**: バージョン番号、日付、著者情報の一括管理
2. **多言語対応**: フロントマターで言語を指定し、コンテンツを切り替え
3. **条件付きコンテンツ**: ターゲット環境（開発/本番）に応じた表示
4. **再利用可能なスニペット**: 頻出するコンテンツをマクロ化

### 未解決の課題

- **パフォーマンス**: 大量の変数展開・マクロ処理の最適化
- **エラーハンドリング**: 未定義変数、構文エラーの報告方法
- **ツールサポート**: エディタでのシンタックスハイライト、補完機能
- **キャッシング戦略**: テンプレート展開結果のキャッシュ方法
- **型システム**: 変数の型チェック（文字列、数値、配列など）

---

## 未実装機能（提案段階）

以下の機能は仕様書で提案されているが、MVP後の追加機能として保留:

- **ラジオボタン**: `( )`, `(x)`
- **トグルボタン**: `< >`, `<x>`
- **画像リンク**: `[![alt](image)](link)`

これらは需要と仕様確定後に実装を検討します。

---

## サポートしない機能

### 絵文字ショートコード

GitHub風の絵文字ショートコード構文（`:thumbsup:`, `:smile:`など）は**サポートしません**。

**理由**:

- 変換テーブルの保守コストが高い
- エンコーディングの透明性を損なう
- 標準的なMarkdown仕様の範囲外

**推奨される代替方法**:

Universal MarkdownはUTF-8/Unicodeエンコーディングを要求するため、Unicode絵文字を直接入力してください：

```markdown
👍 いいね！
😊 笑顔
🎉 お祝い
```

モダンなエディタはすべてUnicode絵文字の入力をサポートしており、OSの絵文字ピッカー（macOS: Ctrl+Cmd+Space、Windows: Win+.）を使用できます。
