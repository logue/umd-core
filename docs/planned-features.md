# 実装予定機能リファレンス

**最終更新**: 2026年5月18日

このドキュメントは、未実装または提案段階の機能のみを記載します。
実装済み機能は [implemented-features.md](implemented-features.md) と各テーマ別ドキュメントを参照してください。

## 目次

- [整理方針](#整理方針)
- [標準プラグイン書式の拡張（未実装）](#標準プラグイン書式の拡張未実装)
- [Markdown拡張機能（検討中）](#markdown拡張機能検討中)
- [セキュリティ設定オプション（提案）](#セキュリティ設定オプション提案)
- [テンプレートエンジン機能（将来構想）](#テンプレートエンジン機能将来構想)
- [未実装機能（提案段階）](#未実装機能提案段階)
- [保留項目](#保留項目)
- [サポートしない機能](#サポートしない機能)

---

## 整理方針

- この文書から実装済み詳細は削除し、リンク参照に統一する。
- 実装済み機能は以下に集約する。
  - [implemented-features.md](implemented-features.md)
  - [umd-extensions.md](umd-extensions.md)
  - [plugin-system.md](plugin-system.md)
  - [media-tags.md](media-tags.md)
  - [table-features.md](table-features.md)
  - [basic-markdown-features.md](basic-markdown-features.md)

### 標準プラグイン書式の扱い

標準プラグイン書式（`@fn(...)`, `&fn(...)`）に属する機能のうち、以下は実装済みのため本ドキュメントの対象外とする。

- `@math(...)` / `&math(...)`
- `@popover(...)` / `&popover(...)`

これらの仕様は [umd-extensions.md](umd-extensions.md) と [plugin-system.md](plugin-system.md) を参照。

---

## 標準プラグイン書式の拡張（未実装）

> 📝 **検討中**

### Dialog プラグイン

ブロック型:

```umd
@dialog(開く){{
  # ダイアログタイトル
  本文
}}
```

インライン型:

```umd
&dialog(開く){本文};
```

出力イメージ:

```html
<button command="show-modal" commandfor="umd-dialog-xxxx">開く</button>
<dialog id="umd-dialog-xxxx">本文</dialog>
```

### Hover/Tooltip プラグイン

ブロック型:

```umd
@hover(対象){{
  補足説明
}}
```

インライン型:

```umd
&hover(補足){対象};
```

出力イメージ:

```html
<span aria-describedby="umd-hover-xxxx">対象</span>
<div role="tooltip" id="umd-hover-xxxx" hidden>補足</div>
```

共通方針:

- ID 生成は `umd-{type}-{uuid}` を採用
- HTML 標準 API と ARIA を優先
- フレームワーク非依存

---

## Markdown拡張機能（検討中）

> 📝 **検討中**

- 仕様の互換性と既存実装への影響を確認しながら段階導入する。
- 実装候補は [PLAN.md](../PLAN.md) の優先順位に従う。

---

## セキュリティ設定オプション（提案）

> 🧪 **提案段階**

### `file:` スキーム許可オプション

現状は `file:` を既定でブロックしている。スタンドアロン用途向けに、明示的 opt-in で許可する設定を検討する。

検討観点:

- 既定値は安全側（拒否）を維持
- 許可時の適用範囲（ローカルのみ、特定コンテキストのみ）
- ドキュメント上の安全注意を必須化

---

## テンプレートエンジン機能（将来構想）

> 🚧 **将来構想**

詳細仕様は [template-engine-spec.md](template-engine-spec.md) を参照。

---

## 未実装機能（提案段階）

> 🧪 **提案段階**

- ラジオボタン: `( )`, `(x)`
- トグルボタン: `< >`, `<x>`
- 画像リンク: `[![alt](image)](link)`

---

## 保留項目

> 🧪 **保留**

### 参照スタイルリンク

```markdown
[ref-link][id]

[id]: https://example.com
```

理由:

- UMD 独自構文との競合リスクがある
- 導入メリットと実装コストの再評価が必要

---

## サポートしない機能

> ⛔ **非対応**

- Setext見出し
- インライン数式（`$...$`）
- 絵文字ショートコード（`:smile:` 形式）
