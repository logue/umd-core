# DPUB-ARIA 対応仕様

**最終更新**: 2026年9月9日

このドキュメントは UMD の [Digital Publishing WAI-ARIA Module (DPUB-ARIA)](https://www.w3.org/TR/dpub-aria-1.1/) への対応方針と、現時点で仕様化するロールを定義します。

## 1. 方針

UMD は将来的に DPUB-ARIA 1.1 準拠を目標としますが、全ロールを一度に実装することはしません。UMD の構文と対応関係が明確なロールから順次追加します。未対応ロールはホストアプリが独自に付与できます。

---

## 2. 仕様化済みロール

### 2.1 `doc-pagebreak` — ページブレーク

物理的なページ区切りを表すマーカーです。印刷・EPUB出力でのページ番号対応を想定しています。

**構文**

```markdown
--- pagebreak: <ラベル> ---
```

`---` の後ろに `pagebreak:` キーワードとラベルを続けます。ラベルは任意の文字列（数字・ローマ数字など）です。

**例**

```markdown
--- pagebreak: 42 ---
---

pagebreak: iv ---
--- pagebreak: A-1 ---
```

**出力HTML**

```html
<hr role="doc-pagebreak" aria-label="42" />
<hr role="doc-pagebreak" aria-label="iv" />
<hr role="doc-pagebreak" aria-label="A-1" />
```

`pagebreak:` なしの `---` は通常の `<hr>` を出力します（CommonMark準拠）。

---

### 2.2 `doc-noteref` / `doc-footnote` — 脚注

CommonMark の脚注構文（`[^n]`）を DPUB-ARIA ロール付きで出力します。

**構文**（変更なし）

```markdown
本文中の参照[^1]。

[^1]: 脚注の内容。
```

**出力HTML**

```html
<!-- インライン参照 -->
<a href="#fn-1" id="fnref-1" role="doc-noteref" aria-label="脚注 1">1</a>

<!-- 脚注本体 -->
<aside id="fn-1" role="doc-footnote">
  <p>脚注の内容。</p>
</aside>
```

脚注セクション全体を囲む要素には `role="doc-footnotes"` を付与します。

---

### 2.3 `doc-notice` / `doc-tip` / `doc-example` — コールアウトブロック

GFM Alert 風のコールアウトブロック（`> [!TYPE]`）に付与します。

詳細は [`umd-extensions.md`](umd-extensions.md#ノート--コールアウトブロック) を参照してください。

| UMDタイプ         | DPUBロール    |
| ----------------- | ------------- |
| `NOTE`・`SUCCESS` | `doc-notice`  |
| `TIP`・`INFO`     | `doc-tip`     |
| `EXAMPLE`         | `doc-example` |

---

## 3. 将来対応予定ロール

以下はUMD構文との対応が想定されるロールです。構文・出力の詳細は未確定です。

| ロール                                            | 説明                       | 対応想定                                   |
| ------------------------------------------------- | -------------------------- | ------------------------------------------ |
| `doc-backlink`                                    | 脚注から参照元に戻るリンク | 脚注出力に追加                             |
| `doc-endnote` / `doc-endnotes`                    | 文末注                     | ブロック型プラグインで対応予定             |
| `doc-biblioref` / `doc-bibliography`              | 参考文献参照・一覧         | ブロック型プラグインで対応予定             |
| `doc-glossterm` / `doc-glossdef` / `doc-glossary` | 用語・定義・用語集         | 定義リスト（`:term\|def`）への適用を検討   |
| `doc-toc`                                         | 目次                       | `&outline()` プラグイン出力に適用予定      |
| `doc-index`                                       | 索引                       | プラグインで対応予定                       |
| `doc-subtitle`                                    | 副題                       | フロントマター `subtitle` フィールドと連動 |
| `doc-abstract`                                    | 要旨                       | コールアウトブロック拡張または専用構文     |
| `doc-pullquote`                                   | プルクォート               | ブロック引用拡張で検討                     |
| `doc-cover`                                       | カバー画像                 | メディアタグ拡張で検討                     |

DPUB-ARIA 1.1 のロール全体については [W3C仕様](https://www.w3.org/TR/dpub-aria-1.1/) を参照してください。
