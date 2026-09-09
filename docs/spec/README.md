# UMD 言語仕様

**最終更新**: 2026年9月9日

このディレクトリには Universal Markdown (UMD) の**言語仕様**を収録しています。

## 対象読者

- VSCode拡張・エディタプラグインの実装者
- ドキュメントサイト・チートシートの制作者
- サードパーティのUMDパーサー実装者
- UMD構文を理解したいユーザー

Rust実装の内部設計については [`../architecture.md`](../architecture.md) を参照してください。

---

## 構文仕様

| ドキュメント | 内容 |
|---|---|
| [basic-markdown-features.md](basic-markdown-features.md) | UMDがサポートするCommonMark構文の一覧 |
| [umd-extensions.md](umd-extensions.md) | UMD独自の拡張構文（強調・装飾・属性など） |
| [comment-syntax.md](comment-syntax.md) | コメント構文 |
| [inline-plugins.md](inline-plugins.md) | インライン型プラグイン（`&func(){}`記法） |
| [block-plugins.md](block-plugins.md) | ブロック型プラグイン（`:::` 記法） |
| [plugin-system.md](plugin-system.md) | プラグインシステム全体の概要・プロトコル |
| [table-features.md](table-features.md) | テーブル構文（GFM拡張・UMDセル連結） |
| [code-block-specification.md](code-block-specification.md) | コードブロック構文・言語指定 |
| [media-tags.md](media-tags.md) | メディアタグ（画像・動画・音声・ダウンロード）の変換仕様 |

## ファイル形式・出力仕様

| ドキュメント | 内容 |
|---|---|
| [file-format.md](file-format.md) | `.umd`・`.umdt`・`.umdx` のファイル形式仕様（フロントマター・ボトムマター） |
| [dpub-aria.md](dpub-aria.md) | DPUB-ARIA 対応仕様（`doc-pagebreak`・`doc-footnote` など） |
| [template-engine-spec.md](template-engine-spec.md) | `.umdt` テンプレートエンジン仕様 |
| [reference-css.md](reference-css.md) | リファレンスCSSクラス（`umd-*`）一覧 |

---

## 設計原則

1. **表記の一意性**: 同じ意味を表す構文を複数用意しない
2. **CommonMark準拠**: 標準Markdownとの互換性を可能な限り維持する
3. **実装非依存**: 本仕様はRust実装に依存しない。他言語での実装を妨げない
