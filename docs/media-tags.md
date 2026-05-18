# メディアタグ自動検出

**最終更新**: 2026年5月18日

画像記法 `![alt](url)` から、拡張子ベースでメディア HTML に変換する仕様です。

## 概要

- 実装: `src/extensions/media.rs`
- 適用フェーズ: 拡張処理（HTML 後処理）
- 主な入力: `<img src="..." alt="..." title="..." />`
- 主なテスト: `src/extensions/media.rs` 内テスト、`tests/bootstrap_integration.rs`

## 変換先

### 動画

対象拡張子:
`.mp4`, `.webm`, `.ogv`, `.mov`, `.avi`, `.mkv`, `.m4v`

出力:

```html
<video controls title="...">
  <source src="..." type="video/..." />
  <track kind="captions" label="..." />
  <a href="..." download class="download-link video-fallback">...</a>
</video>
```

### 音声

対象拡張子:
`.mp3`, `.wav`, `.ogg`, `.oga`, `.m4a`, `.aac`, `.flac`, `.opus`, `.weba`

出力:

```html
<audio controls title="...">
  <source src="..." type="audio/..." />
  <a href="..." download class="download-link audio-fallback">...</a>
</audio>
```

### 画像

対象拡張子:
`.jpg`, `.jpeg`, `.png`, `.gif`, `.svg`, `.webp`, `.avif`, `.bmp`, `.ico`, `.jxl`, `.tif`, `.tiff`

出力:

```html
<picture title="...">
  <source srcset="..." type="image/..." />
  <img src="..." alt="..." loading="lazy" class="img-fluid" title="..." />
</picture>
```

### ダウンロードリンク

対象拡張子:

- アーカイブ: `.zip`, `.tar`, `.gz`, `.7z`, `.rar`, `.bz2`, `.xz`
- ドキュメント: `.pdf`, `.doc`, `.docx`, `.xls`, `.xlsx`, `.ppt`, `.pptx`, `.odt`, `.ods`, `.odp`
- テキスト系: `.txt`, `.md`, `.csv`, `.json`, `.xml`, `.yaml`, `.yml`, `.toml`
- 実行ファイル系: `.exe`, `.dmg`, `.deb`, `.rpm`, `.app`, `.apk`, `.msi`

出力:

```html
<a href="..." download class="download-link" title="...">...</a>
```

## 表示ルール

- 段落がメディア要素だけで構成される場合、`<figure class="w-100">...</figure>` にラップします。
- 段落内のインラインメディアはそのままインライン扱いです。

## オプション

### `allow_fragment_extension_hint`

`ParserOptions.allow_fragment_extension_hint`（既定: `false`）を有効にすると、拡張子なし URL の `#.png` のようなフラグメントヒントをメディア判定に使います。

例:

- `/assets/image#.png` → 画像として判定（`true` の場合）
- `/assets/media#t=10` → 拡張子ヒントではないため判定しない

### `icons`

`ParserOptions.icons` で、動画・音声・ダウンロード・カラーサンプルのアイコン HTML を差し替え可能です。

既定値は Bootstrap Icons ベースです。

## 補足

- クエリ文字列とフラグメントを除いたパス拡張子で判定します（ヒント有効時を除く）。
- 未知拡張子はメディア変換せず、通常の画像 `<picture><img ...></picture>` として扱います。
- alt が空のときは、フォールバックリンクの表示文字に URL を使用します。
