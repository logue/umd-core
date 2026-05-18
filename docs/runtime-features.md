# 実行時機能（フロントマター・脚注・出力）

**最終更新**: 2026年5月18日

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

## カスタムヘッダーID

- `# Title {#custom-id}` をサポート
- 未指定時は自動採番 ID を付与

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
