# コードブロック仕様

コードブロックのメタデータ処理（言語クラスのパススルー、ファイル名からの`<figcaption>`生成、`<figure class="umd-code-block">`ラップ）については [docs/code-block-extensions.md](../code-block-extensions.md) を参照してください。

> **旧仕様について**: 本ファイルは以前、Rust側で完結するシンタックスハイライト（syntect）とMermaid図のSVGレンダリング（mermaid-rs-renderer）の実装仕様を記載していましたが、これらはumd-coreから削除され、ホストアプリケーション（Layer 2/3）の責務になりました。経緯は `CHANGELOG.md` を参照してください。
