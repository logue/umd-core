# WASM Build Instructions

Universal Markdown (UMD) はWebAssemblyとしてビルドできます。

## 前提条件

```bash
# wasm-packのインストール
cargo install wasm-pack
```

## ビルド方法

### リリースビルド（本番用）

```bash
./build.sh release
# または
./build.sh
```

### 開発ビルド（デバッグ用）

```bash
./build.sh dev
```

## 生成されるファイル

`pkg/`ディレクトリに以下のファイルが生成されます：

- `umd.js` - JavaScriptバインディング
- `umd.d.ts` - TypeScript型定義
- `umd_bg.wasm` - WASMバイナリ
- `package.json` - npmパッケージ情報

## 使用例

### ブラウザから直接使用

```html
<!DOCTYPE html>
<html>
  <head>
    <meta charset="utf-8" />
    <title>Universal Markdown Demo</title>
  </head>
  <body>
    <textarea id="input" rows="10" cols="50">
# Hello World

This is **bold** and *italic* text.
    </textarea>
    <button onclick="parseWiki()">Parse</button>
    <div id="output"></div>

    <script type="module">
      import init, { parse_markdown } from "./pkg/umd.js";

      await init();

      window.parseWiki = function () {
        const input = document.getElementById("input").value;
        const html = parse_markdown(input);
        document.getElementById("output").innerHTML = html;
      };
    </script>
  </body>
</html>
```

### Node.jsから使用

```javascript
import init, { parse_markdown } from "./pkg/umd.js";

async function main() {
  await init();

  const input = `
# UMD Example

This is **bold** and *italic* text.

- List item 1
- List item 2
`;

  const html = parse_markdown(input);
  console.log(html);
}

main();
```

### TypeScript

```typescript
import init, { parse_markdown } from "./pkg/umd.js";

async function parseMarkdown(source: string): Promise<string> {
  await init();
  return parse_markdown(source);
}

// 使用例
const html = await parseMarkdown("# Hello World");
console.log(html);
```

## パフォーマンス最適化

リリースビルドでは以下の最適化が適用されます：

- **サイズ最適化** (`opt-level = "z"`)
- **Link Time Optimization** (`lto = true`)
- **単一コード生成ユニット** (`codegen-units = 1`)

これにより、WASMバイナリサイズが最小化され、実行速度も向上します。

## トラブルシューティング

### wasm-packが見つからない

```bash
cargo install wasm-pack
```

### ビルドエラーが発生する

```bash
# 依存関係を更新
cargo update

# クリーンビルド
cargo clean
./build.sh
```
