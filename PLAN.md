# Universal Markdown (UMD) 実装プラン

**プロジェクト概要**: Markdownを超える次世代マークアップ言語。CommonMark仕様テスト 75%+ パス、Bootstrap 5統合、セマンティックHTML、拡張可能なプラグインシステム提供。

**作成日**: 2026年1月23日  
**最終更新**: 2026年2月26日  
**Rustバージョン**: 1.93.1 (Edition 2024)  
**ライセンス**: MIT

---

## ドキュメント構成

このPLAN.mdは**実装状況とロードマップ**を記載しています。詳細は以下を参照：

- **[docs/README.md](docs/README.md)** - ドキュメント索引（入口）
- **[docs/architecture.md](docs/architecture.md)** - システムアーキテクチャ、開発者ガイド、技術設計
- **[docs/implemented-features.md](docs/implemented-features.md)** - 実装済み機能の完全リファレンス
- **[docs/planned-features.md](docs/planned-features.md)** - 実装予定機能の詳細仕様
- **[README.md](README.md)** - プロジェクト概要とユーザー向けドキュメント

---

## プロジェクト現状サマリー

### ✅ 達成済み

- ✅ CommonMark 75%+ 準拠達成
- ✅ Bootstrap 5 統合（Core UI互換）
- ✅ セマンティックHTML生成
- ✅ メディア自動検出（動画・音声・画像・ダウンロード）
- ✅ プラグインシステム（インライン & ブロック型）
- ✅ テーブル拡張（セル連結、配置、装飾）
- ✅ UMD独自構文全体実装
- ✅ セキュリティ対策完備（XSS/URL sanitization）

### 🚧 進行中

- 🚧 ドキュメント整備と最適化
- 🚧 パフォーマンス測定・改善

### 🔮 計画中

- 🔮 数式サポート（LaTeX → MathML）
- 🔮 高度なシンタックスハイライト（ハイブリッド方式）
- 🔮 Mermaid SVG レンダリング（バックエンド最適化）

---

## テスト結果

**総テスト数**: 284 tests passing ✅

```text
193 unit tests ..................... lib.rs
 22 bootstrap integration tests .... 拡張機能
 18 commonmark compliance tests .... 仕様準拠
 14 comment syntax tests .......... 特殊構文
 13 conflict resolution tests ...... 競合処理
 15 doctests ...................... ドキュメント
 12 code block tests .............. コード強化
  1 semantic integration test ...... セマンティック
```

---

## 最近の実装（2026年2月）

### 2026年2月24日

#### ドキュメント体系の統一

- `.github/copilot-instructions.md` を簡潔なリファレンスに再設計
- `docs/architecture.md` に開発者向けガイドを統合
- 重複情報を排除し、単一の情報源を確立

### 2026年2月20日-24日

#### メディア機能の最終化

- ブロック vs インライン自動判別実装
- Bootstrap 5 マージン クラス（`ms/me`）採用
- 幅制御を figure レベルに集約
- 右揃え・中央揃え・左揃え・両端揃えプレフィックス対応

**テスト**: 24/24 integration tests passing

### 2026年2月20日

#### コードブロック強化

- Mermaid SVG レンダリング
- 複数行コンテンツ対応
- Bootstrap CSS 変数自動注入
- 言語別シンタックスハイライト対応

**テスト**: 12 code block tests passing

---

## 実装フェーズ

| Phase                   | 状態      | 期間      | 目標                           |
| ----------------------- | --------- | --------- | ------------------------------ |
| 1: MVP (基本機能)       | ✅ 完了   | 1月-2月初 | Markdown + UMD基本             |
| 2: 準拠性向上           | ✅ 完了   | 2月初中   | CommonMark 75%+                |
| 3: 拡張機能             | ✅ 完了   | 2月中     | テーブル・プラグイン完成       |
| 4: 高度なUMD機能        | ✅ 完了   | 2月18日   | リスト内ブロック・タスク・属性 |
| 5: HTML生成・テスト整備 | 🚧 進行中 | 2月24日-  | プロダクション品質             |
| 6: リリース準備         | 🔮 計画中 | 3月+      | 数式・最適化・ドキュメント     |

---

## 実装予定（次フェーズ）

### 短期（1-2週間）

1. **パフォーマンス最適化**
   - 正規表現パターンのキャッシング
   - 大規模ドキュメント処理の並列化検討
   - ベンチマーク測定

2. **ドキュメント完成**
   - Copilot Instructions の言語別バリアント（英語版）
   - API リファレンス自動生成
   - デモサイトの充実

### 中期（3-6週間）

1. **数式サポート** (MathML生成)
   - LaTeX → MathML 変換
   - KaTeX 互換性確保
   - レンダリング検証

2. **シンタックスハイライト** (ハイブリッド)
   - サーバー側: HTML 属性付与
   - フロントエンド: JavaScript オプション
   - Bootstrap CSS 変数カスタムテーマ

### 長期（2ヶ月以降）

1. **Mermaid 図レンダリング**
   - SVG 生成とキャッシュ
   - Bootstrap カラー自動マッピング
   - ダークモード対応

---

## 技術的負債・改善項目

### 優先度：高

1. **ブロック装飾の複合処理最適化**
   - 現状: 各プレフィックスが個別に `<p>` タグ生成
   - 目標: 統一正規表現で1つのタグに統合
   - ファイル: `src/extensions/block_decorations.rs`, `conflict_resolver.rs`

2. **テーブル装飾の統一**
   - セル装飾関数の標準化
   - Bootstrap クラス マッピング効率化

### 優先度：中

1. **WASM バイナリサイズ最適化**
   - LTO (Link Time Optimization) 有効化
   - 不使用機能の削除検討
   - 目標: pkg/ < 200KB

2. **エラーメッセージの改善**
   - ユーザー向けエラーログ実装
   - デバッグモード（verbose）オプション

---

## 仕様確定事項

詳細は [docs/planned-features.md](docs/planned-features.md) を参照。

- ✅ **URL 自動リンク**: `<URL>` 形式のみサポート（裸 URL は非推奨）
- ✅ **URL スキーム**: `javascript:`, `data:`, `vbscript:`, `file:` ブロック
- ✅ **下線構文**: `__text__` → `<u>` (Discord 風)
- ✅ **数式構文**: `&math(LaTeX);` ($ 記号非採用)
- ✅ **フットノート**: JSON 構造化データ出力
- ✅ **絵文字**: Unicode 直接入力推奨、ショートコード非サポート
- ✅ **改行**: `&br;` 明示的タグ（テーブルセル対応）

---

## 参考リソース

- **仕様**: [LukiWiki Rules](https://github.com/logue/LukiWiki-core/blob/master/docs/rules.md)
- **CommonMark**: [仕様書](https://spec.commonmark.org/)
- **GFM**: [GitHub Flavored Markdown](https://github.github.com/gfm/)
- **Bootstrap 5**: [Documentation](https://getbootstrap.com/docs/5.3/)

---

## ライセンス

MIT License
