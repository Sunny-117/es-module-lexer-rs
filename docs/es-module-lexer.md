`es-module-lexer` 本质上是一个**用 C/Go 写的极快的 ES Module 静态分析器**，核心目标只有一件事：

> **在“不执行 JS、不构建 AST”的前提下，快速找出 `import / export` 的位置与信息**

它被 **Vite / Rollup / esbuild / Rspack** 等大量工具使用。

下面我从 **原理 → 算法 → JS ↔ C 通信机制** 三个层面给你拆开讲（偏工程 & 源码视角）。

---

## 一、es-module-lexer 在做什么？

输入：

```js
import foo from './foo.js'
export { bar } from "./bar.js"
export default function () {}
```

输出的是 **结构化的“位置索引”信息**，而不是 AST：

```ts
{
  imports: [
    {
      s: 0,        // import 起始 index
      e: 30,       // import 结束 index
      ss: 12,      // specifier start
      se: 22,      // specifier end
      d: -1        // dynamic import 标志
    }
  ],
  exports: [...]
}
```

⚠️ **它不关心语义，只关心字符串里的 token**

---

## 二、核心原理：为什么不用 AST 还能这么快？

### 1️⃣ 状态机 + 单次线性扫描（O(n)）

es-module-lexer 的核心是一个 **手写状态机（finite-state machine）**：

```text
Normal
 ├── String (' " `)
 ├── Comment (// /* */)
 ├── Regex
 ├── Import
 ├── Export
```

**只扫描一遍字符串，不回溯、不递归**

---

### 2️⃣ 为什么能“只靠字符”识别 import / export？

因为 ES Module 语法满足：

```text
import/export 一定是关键字级别
不能出现在字符串、注释、正则里
```

所以只要正确处理：

* 字符串
* 模板字符串
* 注释
* 正则

就 **可以 100% 静态判断**

---

### 3️⃣ 关键技巧：**不用 AST，只记录 index**

例如：

```js
import(/*comment*/ './foo.js')
```

lexer 会做：

```txt
i m p o r t ( ... )
↑              ↑
s              e
```

而不是解析成：

```ts
ImportExpression {
  source: Literal('./foo.js')
}
```

👉 这就是 **快 10~50 倍** 的根本原因。

---

## 三、es-module-lexer 的底层实现

### 实际结构（简化）

```
es-module-lexer/
├── src/
│   ├── lexer.c        // 核心扫描逻辑（C）
│   ├── wasm.c        // WASM 导出
│   └── lexer.js      // JS 包装
├── dist/
│   └── lexer.wasm
```

> 早期是 C，现在主要是 **C → WASM**

---

## 四、JS 是如何和 C 通信的？（重点）

### 总体方案

> **C → WASM → JS**

也就是说：

```
C 代码
 ↓ 编译
WebAssembly (.wasm)
 ↓ 调用
JavaScript
```

---

## 五、C → WASM 的关键点

### 1️⃣ C 函数导出

在 C 中：

```c
// lexer.c
int parse(const char* input, int len) {
  // 扫描 input
  // 把结果写入共享内存
  return result_count;
}
```

通过 Emscripten 编译：

```bash
emcc lexer.c -O3 \
  -s EXPORTED_FUNCTIONS="['_parse']" \
  -s EXPORTED_RUNTIME_METHODS="['cwrap']" \
  -o lexer.wasm
```

---

### 2️⃣ WASM 线性内存（最关键）

JS 和 C **并不直接传对象**，而是共享一块 **ArrayBuffer**：

```txt
WASM Memory (线性内存)
┌─────────────────────────┐
│ input string bytes      │  ← JS 写
│ import records struct   │  ← C 写
└─────────────────────────┘
```

---

## 六、JS → C：如何传 JS 字符串？

### JS 侧

```js
const memory = wasmInstance.exports.memory
const buffer = new Uint8Array(memory.buffer)

function writeString(str) {
  const ptr = malloc(str.length)
  for (let i = 0; i < str.length; i++) {
    buffer[ptr + i] = str.charCodeAt(i)
  }
  return ptr
}

const ptr = writeString(code)
parse(ptr, code.length)
```

> JS **只传指针 + 长度**

---

## 七、C → JS：如何返回复杂结果？

### C 中写结构体

```c
typedef struct {
  int s;
  int e;
  int ss;
  int se;
  int d;
} Import;
```

写到内存中：

```c
imports[i].s = start;
imports[i].e = end;
```

---

### JS 中读取

```js
const view = new Int32Array(memory.buffer)

const base = importsPtr / 4
const s = view[base]
const e = view[base + 1]
```

👉 **没有 JSON，没有序列化，零拷贝**

---

## 八、为什么不用 Node-API / napi-rs？

你做过 `napi-rs`（我记得你在写 Rust bundler 👍），对比一下：

| 方案       | 优点            | 缺点        |
| -------- | ------------- | --------- |
| **WASM** | 浏览器 + Node 通用 | 调用成本略高    |
| N-API    | Node 原生，调试好   | 无法跑浏览器    |
| napi-rs  | Rust 体验好      | Node-only |

👉 es-module-lexer 的目标是：

> **工具链通用（Vite / 浏览器 / Node / Deno）**

所以选了 WASM。

---

## 九、为什么这套设计特别适合你现在做的事？

你现在在做的项目里有：

* Rust bundler
* import / dependency 分析
* 不想 full AST（性能）

👉 es-module-lexer 的思想可以直接迁移到 **Rust**：

* 用 `bytes + FSM`
* 只扫描 token
* 返回 index
* 上层再 slice 字符串

如果你愿意，我可以：

1. **手写一个 Rust 版 mini es-module-lexer**
2. 或直接对比 `oxc-parser` vs lexer 的性能 trade-off
3. 或画一张 **JS ↔ WASM 内存布局图**

你想从哪一个继续？
