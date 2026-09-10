# 基本Markdown機能

**最終更新**: 2026年5月18日

Universal Markdown が提供する基本 Markdown 機能の実装一覧です。

## 対応範囲

- ATX 見出し (`#` から `######`)
- 段落・改行
- リスト（順序あり/なし）
- 強調（`*text*` → `<em>`, `**text**` → `<strong>`）
- リンク・画像
- 明示的自動リンク（`<https://...>`）
- GFM テーブル
- GFM タスクリスト
- GFM 取り消し線（`~~...~~`）

> [!NOTE]
> 標準CommonMarkでは `__text__` も `**text**` と同じ `<strong>` になりますが、
> UMDでは「複数の書き方に同じ意味を持たせない」という設計方針により
> `__text__` をDiscord風の下線（`<u>`）専用構文として再定義しています。
> 詳細は [architecture.md の設計方針](architecture.md#設計方針)を参照してください。

## 見出しアンカー

見出し行の末尾に `{#custom-id}` を書くと、そのアンカーIDを固定できます。
`id` は見出しタグ（`<h1>`〜`<h6>`）自身の属性として付与されます。

- `{#custom-id}` を指定した場合、そのIDがプレフィックスなしでそのまま使われ
  ます。他の見出しIDと重複してもパーサーはエラーにしません（HTML的には無効
  な重複IDになりますが、その回避はユーザーの責任とし、linterでの検出に委ね
  ます）
- 未指定の見出しは文書内の出現順による自動採番で、他のIDと衝突しないよう
  `h-` プレフィックスが付きます（`h-1`, `h-2`, ...）

```text
Input:  # 見出し１
        ## 見出し２
        ### アンカーつき見出し {#anchor}
        ## 見出し３

Output: <h1 id="h-1">見出し１</h1>
        <h2 id="h-2">見出し２</h2>
        <h3 id="anchor">アンカーつき見出し</h3>
        <h2 id="h-3">見出し３</h2>
```

- ID として使える文字は英数字・`_`・`-` のみ
- 詳細・制約は [runtime-features.md のカスタムヘッダーID](../runtime-features.md#カスタムヘッダーid)を参照

> [!WARNING]
> 現在の実装はこの仕様通りに動作していません。常に `h-` プレフィックスが付き
> （ユーザー指定の`{#custom-id}`にも適用されてしまう）、`id`は`<hN>`タグ自身
> ではなく別途挿入される空の`<a>`要素に付与され、見出し内にインライン記法が
> あるとアンカー自体が失われることもあります。既知の問題として
> [runtime-features.md](../runtime-features.md#既知の問題見出しid処理が設計意図と異なる)
> に詳細を記載しています。

## コードブロック拡張

コードブロックの詳細は以下を参照してください。

- [../code-block-extensions.md](../code-block-extensions.md)

主な機能:

- `language-*` クラスのパススルー（シンタックスハイライトやMermaid/GeoJSON等のレンダリングはホストアプリケーションの責務）
- `lang:filename` 形式のファイル名付きコードブロック
- インラインコードのカラーサンプル

## 実装の主担当

- `src/parser.rs`
- `src/extensions/fence/code_block.rs`（言語クラスのパススルー・ファイル名）
- `src/extensions/fence/normalize.rs`（`lang:filename` 記法の正規化）
- `src/extensions/fence/protect.rs`（コード区間の保護・カラーサンプル検出）
- `src/extensions/conflict_resolver.rs`

## 主なテスト

- `tests/commonmark.rs`
- `tests/rendering_integration.rs`
- `examples/test_output.rs`
- `examples/code_block_extensions.rs`
