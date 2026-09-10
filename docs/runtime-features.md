# 実行時機能（フロントマター・脚注・出力）

**最終更新**: 2026年9月10日

パーサーの入出力と実行時オプションに関する仕様です。

## フロントマター

- YAML (`---`) / TOML (`+++`) を抽出
- HTML 出力には含めない
- `ParseResult.frontmatter` で取得

## 脚注

- CommonMark 脚注を有効化
- `parse_with_frontmatter*` は本文と脚注セクションを分離

```rust
pub struct ParseResult {
    pub html: String,
    pub frontmatter: Option<Frontmatter>,
    pub footnotes: Option<String>,
}
```

- `parse` は `html` + `footnotes` を結合した文字列を返却

> [!NOTE]
> プラグイン構文（`&function(){content};` / `@function(){{content}}`）の
> `content`内に書かれた脚注参照`[^N]`は機能しません。制限事項の詳細は
> [known-issues.md](known-issues.md#プラグイン構文内の脚注参照は非対応制限事項)
> を参照してください。

## カスタムヘッダーID

- `# Title {#custom-id}` をサポート
- 未指定時は自動採番 ID を付与

> [!WARNING]
> 現在の実装には、見出しへのインライン記法（強調・リンク等）併用時に
> アンカーが失われる等、設計意図と異なる既知のバグがあります。詳細・
> 回避策は
> [known-issues.md](known-issues.md#見出しid処理が設計意図と異なる未修正バグ)
> を参照してください。

## Base URL

- `ParserOptions.base_url` で `/path` を自動解決
- 外部 URL は変更しない

## ParserOptions（主なもの）

- `gfm_extensions`
- `umd_extensions`
- `max_heading_level`
- `max_inline_nesting`
- `base_url`
- `allow_fragment_extension_hint`
- `icons`

## 実装の主担当

- `src/lib.rs`
- `src/frontmatter.rs`
- `src/parser.rs`
- `src/extensions/conflict_resolver.rs`

## 主なテスト

- `tests/base_url.rs`
- `examples/test_frontmatter.rs`
- `examples/test_footnotes.rs`
- `examples/test_header_id.rs`

## 既知の問題

このファイルが扱う機能（脚注・見出しID等）に関する既知の問題・制限事項は
[known-issues.md](known-issues.md)にまとめています。プラグイン構文全般の
既知の問題（ネストの扱い等）も同ファイルを参照してください。
