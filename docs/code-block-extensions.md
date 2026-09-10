# コードブロック拡張機能

このドキュメントは、UMDパーサーのコードブロック処理（`src/extensions/fence/code_block.rs`）の実装と使用方法について説明します。

> **破壊的変更（2026年9月〜）**: シンタックスハイライト（旧: syntect）とMermaid図のSVGレンダリング（旧: mermaid-rs-renderer）は umd-core から削除されました。これらは「ドメイン固有のビジュアル表現」であり、3層アーキテクチャ（[docs/architecture.md](architecture.md)参照）の判断基準に照らしてホストアプリケーション（Layer 2/3）の責務とすべき、という結論に基づく変更です。加えて `mermaid-rs-renderer` はOSのフォントディレクトリスキャンに依存する `fontdb` を使っており、そもそも wasm32 ターゲットではビルドできませんでした。旧実装の詳細は `CHANGELOG.md` を参照してください。

## umd-coreが行うこと

umd-coreはコードブロックについて、以下のメタデータ処理のみを行います。

- `language-*` クラスのパススルー（comrakが生成したものをそのまま維持、解釈・変換はしない）
- ファイル名メタデータ（fence info文字列の `lang:filename` 記法、`src/extensions/fence/normalize.rs` が正規化）から `<figcaption>` を生成
- 常に `<figure class="umd-code-block">` でラップ（プレーンテキスト・言語指定ありのどちらも）

言語の直後に `:` を続けるとファイル名として扱われます（`:` より前がなければ言語なしファイル名指定として扱われます）。

```text
Input:  ```rust:main.rs
        fn main() {}
        ```

Output: <figure class="umd-code-block">
          <figcaption class="umd-code-filename">main.rs</figcaption>
          <pre><code class="language-rust">fn main() {}</code></pre>
        </figure>
```

`mermaid`・`geojson`など、umd-coreが特別扱いしない言語も全く同じ経路を通ります。

```text
Input:  ```mermaid
        flowchart TD
          A[Start] --> B[End]
        ```

Output: <figure class="umd-code-block">
          <pre><code class="language-mermaid">flowchart TD
          A[Start] --> B[End]</code></pre>
        </figure>
```

## ホストアプリケーションが行うこと（Layer 2/3）

シンタックスハイライトや、Mermaid/GeoJSONのような言語固有のレンダリングは、`<code class="language-*">` のクラスを検出してホスト側で行います。

- **シンタックスハイライト**: highlight.js / Shiki / Prism.js などをクライアントサイドで適用するか、ビルド時にSSRする
- **Mermaid**: `language-mermaid` を検出し、`mermaid.js`（クライアントサイド）またはビルド時ツール（サーバーサイド/ネイティブ）でSVGを生成し、`<pre><code>` を置き換える
- **GeoJSON**: `language-geojson` を検出し、OpenLayers等で地図に変換する

置き換え時は元の `<pre><code>` ノードを削除せず、`umd-hidden` クラス等で隠しつつDOMに残しておくと、「ソース表示」トグルや、レンダリング失敗時のフォールバック表示に使えます（詳細は各ホストアプリの実装に委ねられます）。

## 関連ファイル

- `src/extensions/fence/code_block.rs`: このドキュメントが説明する処理の実装
- `src/extensions/fence/normalize.rs`: `lang:filename` 記法の正規化（comrakへ渡す前に `lang umd-filename:filename` 形式へ変換）
- [docs/architecture.md](architecture.md): 3層アーキテクチャの判断基準全般
